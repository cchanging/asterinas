// SPDX-License-Identifier: MPL-2.0

#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <linux/nsfs.h>
#include <linux/sched.h>
#include <pthread.h>
#include <sched.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include "../../common/test.h"

#define CGROUP_ROOT "/sys/fs/cgroup"

/*
 * Each test creates its own small cgroup tree under the unified hierarchy.
 * `mount` is used only by the cgroupfs visibility test.
 */
struct cgroup_test_dirs {
	char base[PATH_MAX];
	char nested[PATH_MAX];
	char peer[PATH_MAX];
	char mount[PATH_MAX];
};

enum child_cgroup_ns_mode {
	CHILD_KEEP_CGROUP_NS,
	CHILD_UNSHARE_CGROUP_NS,
};

struct child_config {
	const char *cgroup_dir;
	enum child_cgroup_ns_mode cgroup_ns_mode;
};

/*
 * The worker thread unshares its cgroup namespace and then stays parked until
 * the main thread has inspected both namespace file inodes.
 */
struct thread_cgroup_ns_args {
	pthread_cond_t cond;
	pthread_mutex_t lock;
	pid_t tid;
	int unshare_errno;
	int should_exit;
};

static int join_path(char *buf, size_t size, const char *dir, const char *name)
{
	int len = snprintf(buf, size, "%s/%s", dir, name);

	if (len < 0 || (size_t)len >= size) {
		errno = ENAMETOOLONG;
		return -1;
	}

	return 0;
}

static const char *relative_to_cgroup_root(const char *path)
{
	/*
	 * `/proc/<pid>/cgroup` only prints the virtual path inside cgroupfs, so the
	 * `/sys/fs/cgroup` mount prefix must be removed first.
	 */
	return path + strlen(CGROUP_ROOT);
}

static const char *path_basename(const char *path)
{
	const char *basename = strrchr(path, '/');

	return basename ? basename + 1 : path;
}

static void init_cgroup_test_dirs(struct cgroup_test_dirs *dirs)
{
	snprintf(dirs->base, sizeof(dirs->base), CGROUP_ROOT "/cgns-base-%d",
		 getpid());
	snprintf(dirs->peer, sizeof(dirs->peer), CGROUP_ROOT "/cgns-peer-%d",
		 getpid());
	snprintf(dirs->mount, sizeof(dirs->mount), "/tmp/cgns-mnt-%d",
		 getpid());
	CHECK(join_path(dirs->nested, sizeof(dirs->nested), dirs->base,
			"nested"));
}

static void cleanup_cgroup_test_dirs(const struct cgroup_test_dirs *dirs)
{
	(void)umount(dirs->mount);
	(void)rmdir(dirs->mount);
	(void)rmdir(dirs->nested);
	(void)rmdir(dirs->base);
	(void)rmdir(dirs->peer);
}

static int move_pid_to_cgroup(const char *cgroup_dir, pid_t pid)
{
	char path[PATH_MAX];
	char content[32];
	int fd;

	if (join_path(path, sizeof(path), cgroup_dir, "cgroup.procs") < 0)
		return -1;
	snprintf(content, sizeof(content), "%d", pid);

	fd = open(path, O_WRONLY);
	if (fd < 0)
		return -1;
	if (write(fd, content, strlen(content)) < 0) {
		close(fd);
		return -1;
	}
	close(fd);
	return 0;
}

static int read_proc_cgroup(pid_t pid, char *buf, size_t size)
{
	char path[64];
	int fd;
	ssize_t count;

	snprintf(path, sizeof(path), "/proc/%d/cgroup", pid);

	fd = open(path, O_RDONLY);
	if (fd < 0)
		return -1;
	count = read(fd, buf, size - 1);
	close(fd);
	if (count < 0)
		return -1;

	buf[count] = '\0';
	return 0;
}

static int read_mount_root(const char *mount_point, char *root, size_t size)
{
	FILE *mountinfo = fopen("/proc/self/mountinfo", "r");
	char line[512];

	if (!mountinfo)
		return -1;

	while (fgets(line, sizeof(line), mountinfo)) {
		char parsed_root[256];
		char parsed_mount_point[256];
		char parsed_fs_type[64];

		if (sscanf(line, "%*s %*s %*s %255s %255s %*s - %63s",
			   parsed_root, parsed_mount_point,
			   parsed_fs_type) != 3)
			continue;
		if (strcmp(parsed_mount_point, mount_point) != 0)
			continue;
		if (strcmp(parsed_fs_type, "cgroup2") != 0)
			continue;

		snprintf(root, size, "%s", parsed_root);
		fclose(mountinfo);
		return 0;
	}

	fclose(mountinfo);
	errno = ENOENT;
	return -1;
}

static int open_task_cgroup_ns_fd(pid_t tid)
{
	char path[64];

	snprintf(path, sizeof(path), "/proc/self/task/%d/ns/cgroup", tid);

	return open(path, O_RDONLY);
}

static pid_t spawn_child(const struct child_config *config, int *release_fd_out)
{
	int ready_pipe[2];
	int release_pipe[2];
	char pipe_byte;
	pid_t child;

	if (pipe(ready_pipe) < 0)
		return -1;
	if (pipe(release_pipe) < 0) {
		close(ready_pipe[0]);
		close(ready_pipe[1]);
		return -1;
	}

	child = fork();
	if (child < 0) {
		close(ready_pipe[0]);
		close(ready_pipe[1]);
		close(release_pipe[0]);
		close(release_pipe[1]);
		return -1;
	}

	if (child == 0) {
		close(ready_pipe[0]);
		close(release_pipe[1]);

		/*
		 * The child first moves to the requested cgroup, then optionally
		 * snapshots that location as the root of a new cgroup namespace.
		 */
		if (config->cgroup_dir &&
		    move_pid_to_cgroup(config->cgroup_dir, getpid()) < 0)
			_exit(2);
		if (config->cgroup_ns_mode == CHILD_UNSHARE_CGROUP_NS &&
		    unshare(CLONE_NEWCGROUP) < 0)
			_exit(3);
		if (write(ready_pipe[1], "", 1) != 1)
			_exit(4);

		close(ready_pipe[1]);
		if (read(release_pipe[0], &pipe_byte, 1) != 1)
			_exit(5);
		close(release_pipe[0]);
		_exit(0);
	}

	close(ready_pipe[1]);
	close(release_pipe[0]);
	if (read(ready_pipe[0], &pipe_byte, 1) != 1) {
		if (errno == 0)
			errno = EIO;
		close(ready_pipe[0]);
		close(release_pipe[1]);
		return -1;
	}

	close(ready_pipe[0]);
	*release_fd_out = release_pipe[1];

	return child;
}

static int wait_for_child_exit(pid_t pid)
{
	int status;

	if (waitpid(pid, &status, 0) != pid) {
		if (errno == 0)
			errno = ECHILD;
		return -1;
	}
	if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
		errno = ECHILD;
		return -1;
	}

	return 0;
}

static void *unshare_cgroup_ns_thread(void *arg)
{
	struct thread_cgroup_ns_args *thread_args = arg;
	int unshare_errno = 0;

	CHECK_WITH(pthread_mutex_lock(&thread_args->lock), _ret == 0);
	thread_args->tid = syscall(SYS_gettid);
	CHECK_WITH(pthread_mutex_unlock(&thread_args->lock), _ret == 0);

	if (unshare(CLONE_NEWCGROUP) < 0)
		unshare_errno = errno;

	CHECK_WITH(pthread_mutex_lock(&thread_args->lock), _ret == 0);
	thread_args->unshare_errno = unshare_errno;
	CHECK_WITH(pthread_cond_signal(&thread_args->cond), _ret == 0);
	while (unshare_errno == 0 && !thread_args->should_exit)
		CHECK_WITH(pthread_cond_wait(&thread_args->cond,
					     &thread_args->lock),
			   _ret == 0);
	CHECK_WITH(pthread_mutex_unlock(&thread_args->lock), _ret == 0);

	return NULL;
}

/*
 * `unshare(CLONE_NEWCGROUP)` is thread-scoped. The worker should get a new
 * namespace inode, while the main thread keeps the original one.
 */
FN_TEST(cgroup_ns_is_thread_local)
{
	struct thread_cgroup_ns_args thread_args = {
		.cond = PTHREAD_COND_INITIALIZER,
		.lock = PTHREAD_MUTEX_INITIALIZER,
		.unshare_errno = -1,
	};
	pthread_t worker_thread;
	struct stat main_before_stat;
	struct stat main_after_stat;
	struct stat worker_stat;
	int main_before_fd;
	int main_after_fd;
	int worker_fd;
	int worker_ready;
	pid_t main_tid = TEST_SUCC(syscall(SYS_gettid));
	pid_t worker_tid = 0;

	main_before_fd = TEST_SUCC(open_task_cgroup_ns_fd(main_tid));
	TEST_SUCC(fstat(main_before_fd, &main_before_stat));

	TEST_RES(pthread_create(&worker_thread, NULL, unshare_cgroup_ns_thread,
				&thread_args),
		 _ret == 0);

	TEST_RES(pthread_mutex_lock(&thread_args.lock), _ret == 0);
	while (thread_args.unshare_errno < 0)
		TEST_RES(pthread_cond_wait(&thread_args.cond,
					   &thread_args.lock),
			 _ret == 0);
	worker_tid = thread_args.tid;
	worker_ready = thread_args.unshare_errno == 0;
	TEST_RES(pthread_mutex_unlock(&thread_args.lock), _ret == 0);
	TEST_RES(0, worker_ready);

	if (worker_ready) {
		main_after_fd = TEST_SUCC(open_task_cgroup_ns_fd(main_tid));
		worker_fd = TEST_SUCC(open_task_cgroup_ns_fd(worker_tid));
		TEST_SUCC(fstat(main_after_fd, &main_after_stat));
		TEST_SUCC(fstat(worker_fd, &worker_stat));

		TEST_RES(0, main_before_stat.st_ino == main_after_stat.st_ino);
		TEST_RES(0, main_after_stat.st_ino != worker_stat.st_ino);

		TEST_RES(pthread_mutex_lock(&thread_args.lock), _ret == 0);
		thread_args.should_exit = 1;
		TEST_RES(pthread_cond_signal(&thread_args.cond), _ret == 0);
		TEST_RES(pthread_mutex_unlock(&thread_args.lock), _ret == 0);

		TEST_SUCC(close(main_after_fd));
		TEST_SUCC(close(worker_fd));
	}

	TEST_RES(pthread_join(worker_thread, NULL), _ret == 0);
	TEST_SUCC(close(main_before_fd));
	TEST_RES(pthread_cond_destroy(&thread_args.cond), _ret == 0);
	TEST_RES(pthread_mutex_destroy(&thread_args.lock), _ret == 0);
}
END_TEST()

/*
 * `/proc/[pid]/cgroup` is rendered relative to the caller's active cgroup
 * namespace. This test covers three views:
 *   1. the original namespace, where `/base` and `/peer` are both visible;
 *   2. a new namespace rooted at `/base`, where self becomes `/` and `peer`
 *      is shown as `/../peer`;
 *   3. the same new namespace after moving self back to the real cgroup root,
 *      where self is now outside the namespace root and becomes `/..`.
 */
FN_TEST(proc_cgroup_is_relative_to_callers_ns)
{
	struct cgroup_test_dirs dirs;
	struct child_config peer_child_config = {
		.cgroup_dir = NULL,
		.cgroup_ns_mode = CHILD_KEEP_CGROUP_NS,
	};
	char proc_buf[PATH_MAX];
	char expected[PATH_MAX];
	int old_nsfd;
	int new_nsfd;
	int peer_release_fd;
	pid_t peer_child;

	init_cgroup_test_dirs(&dirs);
	cleanup_cgroup_test_dirs(&dirs);

	TEST_SUCC(mkdir(dirs.base, 0755));
	TEST_SUCC(mkdir(dirs.nested, 0755));
	TEST_SUCC(mkdir(dirs.peer, 0755));

	old_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));
	TEST_RES(ioctl(old_nsfd, NS_GET_NSTYPE), _ret == CLONE_NEWCGROUP);

	TEST_SUCC(move_pid_to_cgroup(dirs.base, getpid()));
	TEST_RES(snprintf(expected, sizeof(expected), "0::%s\n",
			  relative_to_cgroup_root(dirs.base)),
		 _ret >= 0 && (size_t)_ret < sizeof(expected));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, expected) == 0);

	peer_child_config.cgroup_dir = dirs.peer;
	peer_child =
		TEST_SUCC(spawn_child(&peer_child_config, &peer_release_fd));
	TEST_RES(snprintf(expected, sizeof(expected), "0::%s\n",
			  relative_to_cgroup_root(dirs.peer)),
		 _ret >= 0 && (size_t)_ret < sizeof(expected));
	TEST_RES(read_proc_cgroup(peer_child, proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, expected) == 0);

	TEST_SUCC(unshare(CLONE_NEWCGROUP));
	new_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));
	/* The namespace is rooted at the caller's current cgroup: `dirs.base`. */
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/\n") == 0);

	/* A sibling cgroup is still reachable, but only via `..` escape. */
	TEST_RES(snprintf(expected, sizeof(expected), "0::/../%s\n",
			  path_basename(dirs.peer)),
		 _ret >= 0 && (size_t)_ret < sizeof(expected));
	TEST_RES(read_proc_cgroup(peer_child, proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, expected) == 0);

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_SUCC(move_pid_to_cgroup(CGROUP_ROOT, getpid()));
	/* Back in the initial namespace, the real hierarchy root is `/`. */
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/\n") == 0);

	TEST_SUCC(setns(new_nsfd, 0));
	/*
	 * Now the caller sits above the namespace root (`/base`), so Linux
	 * renders the current cgroup as `/..`.
	 */
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/..\n") == 0);

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_SUCC(move_pid_to_cgroup(CGROUP_ROOT, getpid()));
	TEST_RES(write(peer_release_fd, "", 1), _ret == 1);
	TEST_SUCC(close(peer_release_fd));
	TEST_SUCC(wait_for_child_exit(peer_child));
	TEST_SUCC(close(new_nsfd));
	TEST_SUCC(close(old_nsfd));
	cleanup_cgroup_test_dirs(&dirs);
}
END_TEST()

/*
 * `setns()` should let the caller join another task's cgroup namespace.
 * After joining, `/proc/self/cgroup` must be re-virtualized against the new
 * namespace root and collapse back to `/`.
 */
FN_TEST(setns_can_join_cgroup_ns)
{
	struct cgroup_test_dirs dirs;
	struct child_config child_ns_config = {
		.cgroup_dir = NULL,
		.cgroup_ns_mode = CHILD_UNSHARE_CGROUP_NS,
	};
	char proc_buf[PATH_MAX];
	char expected[PATH_MAX];
	int old_nsfd;
	int new_nsfd;
	int pidfd;
	int child_ns_release_fd;
	pid_t child_ns;

	init_cgroup_test_dirs(&dirs);
	cleanup_cgroup_test_dirs(&dirs);

	TEST_SUCC(mkdir(dirs.base, 0755));

	old_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));

	TEST_SUCC(move_pid_to_cgroup(dirs.base, getpid()));
	TEST_SUCC(unshare(CLONE_NEWCGROUP));
	new_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/\n") == 0);

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_RES(snprintf(expected, sizeof(expected), "0::%s\n",
			  relative_to_cgroup_root(dirs.base)),
		 _ret >= 0 && (size_t)_ret < sizeof(expected));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, expected) == 0);

	TEST_SUCC(setns(new_nsfd, 0));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/\n") == 0);

	TEST_SUCC(setns(old_nsfd, 0));
	child_ns =
		TEST_SUCC(spawn_child(&child_ns_config, &child_ns_release_fd));

	/* Join a child whose namespace root is its current cgroup. */
	pidfd = TEST_SUCC(syscall(SYS_pidfd_open, child_ns, 0));
	TEST_SUCC(setns(pidfd, CLONE_NEWCGROUP));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/\n") == 0);

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_SUCC(move_pid_to_cgroup(CGROUP_ROOT, getpid()));
	TEST_SUCC(close(pidfd));
	TEST_RES(write(child_ns_release_fd, "", 1), _ret == 1);
	TEST_SUCC(close(child_ns_release_fd));
	TEST_SUCC(wait_for_child_exit(child_ns));
	TEST_SUCC(close(new_nsfd));
	TEST_SUCC(close(old_nsfd));
	cleanup_cgroup_test_dirs(&dirs);
}
END_TEST()

/*
 * A fresh `cgroup2` mount inside a cgroup namespace should expose that
 * namespace's root as `/`. Descendants below the root remain reachable, while
 * paths above the root must disappear from the mount.
 */
FN_TEST(cgroup_mount_hides_paths_above_ns_root)
{
	struct cgroup_test_dirs dirs;
	char proc_buf[PATH_MAX];
	char mount_root[PATH_MAX];
	char visible_path[PATH_MAX];
	char hidden_path[PATH_MAX];
	int old_nsfd;
	int new_nsfd;

	init_cgroup_test_dirs(&dirs);
	cleanup_cgroup_test_dirs(&dirs);

	TEST_SUCC(mkdir(dirs.base, 0755));
	TEST_SUCC(mkdir(dirs.nested, 0755));

	old_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));

	TEST_SUCC(move_pid_to_cgroup(dirs.base, getpid()));
	TEST_SUCC(unshare(CLONE_NEWCGROUP));
	new_nsfd = TEST_SUCC(open("/proc/self/ns/cgroup", O_RDONLY));

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_SUCC(move_pid_to_cgroup(CGROUP_ROOT, getpid()));
	TEST_SUCC(setns(new_nsfd, 0));
	TEST_RES(read_proc_cgroup(getpid(), proc_buf, sizeof(proc_buf)),
		 strcmp(proc_buf, "0::/..\n") == 0);

	TEST_SUCC(unshare(CLONE_NEWNS));
	TEST_SUCC(mkdir(dirs.mount, 0755));
	TEST_SUCC(mount("none", dirs.mount, "cgroup2", 0, NULL));
	/* The mounted filesystem root is the namespace root, not the global one. */
	TEST_RES(read_mount_root(dirs.mount, mount_root, sizeof(mount_root)),
		 strcmp(mount_root, "/") == 0);

	TEST_SUCC(join_path(visible_path, sizeof(visible_path), dirs.mount,
			    "nested"));
	TEST_SUCC(access(visible_path, F_OK));

	TEST_SUCC(join_path(hidden_path, sizeof(hidden_path), dirs.mount,
			    path_basename(dirs.base)));
	TEST_ERRNO(access(hidden_path, F_OK), ENOENT);

	TEST_SUCC(setns(old_nsfd, 0));
	TEST_SUCC(move_pid_to_cgroup(CGROUP_ROOT, getpid()));
	TEST_SUCC(close(new_nsfd));
	TEST_SUCC(close(old_nsfd));
	cleanup_cgroup_test_dirs(&dirs);
}
END_TEST()
