# SPDX-License-Identifier: MPL-2.0

# =========================== Makefile options. ===============================

# Global build options.
ARCH ?= x86_64
BENCHMARK ?= none
BOOT_METHOD ?= grub-rescue-iso
BOOT_PROTOCOL ?= multiboot2
BUILD_SYSCALL_TEST ?= 0
ENABLE_KVM ?= 1
INTEL_TDX ?= 0
MEM ?= 8G
OVMF ?= on
RELEASE ?= 0
RELEASE_LTO ?= 0
LOG_LEVEL ?= error
SCHEME ?= ""
SMP ?= 1
KSTD_TASK_STACK_SIZE_IN_PAGES ?= 64
FEATURES ?=
NO_DEFAULT_FEATURES ?= 0
# End of global build options.

# GDB debugging and profiling options.
GDB_TCP_PORT ?= 1234
GDB_PROFILE_FORMAT ?= flame-graph
GDB_PROFILE_COUNT ?= 200
GDB_PROFILE_INTERVAL ?= 0.1
# End of GDB options.

# The Makefile provides a way to run arbitrary tests in the kernel
# mode using the kernel command line.
# Here are the options for the auto test feature.
AUTO_TEST ?= none
EXTRA_BLOCKLISTS_DIRS ?= ""
SYSCALL_TEST_DIR ?= /tmp
# End of auto test features.

# Network settings
# NETDEV possible values are user,tap
NETDEV ?= user
VHOST ?= off
# End of network settings

# ========================= End of Makefile options. ==========================

CARGO_KSDK := ~/.cargo/bin/cargo-ksdk

CARGO_KSDK_ARGS := --target-arch=$(ARCH) --kcmd-args="kstd.log_level=$(LOG_LEVEL)"

ifeq ($(AUTO_TEST), syscall)
BUILD_SYSCALL_TEST := 1
CARGO_KSDK_ARGS += --kcmd-args="SYSCALL_TEST_DIR=$(SYSCALL_TEST_DIR)"
CARGO_KSDK_ARGS += --kcmd-args="EXTRA_BLOCKLISTS_DIRS=$(EXTRA_BLOCKLISTS_DIRS)"
CARGO_KSDK_ARGS += --init-args="/opt/syscall_test/run_syscall_test.sh"
else ifeq ($(AUTO_TEST), test)
	ifneq ($(SMP), 1)
		CARGO_KSDK_ARGS += --kcmd-args="BLOCK_UNSUPPORTED_SMP_TESTS=1"
	endif
CARGO_KSDK_ARGS += --init-args="/test/run_general_test.sh"
else ifeq ($(AUTO_TEST), boot)
CARGO_KSDK_ARGS += --init-args="/test/boot_hello.sh"
else ifeq ($(AUTO_TEST), vsock)
export VSOCK=on
CARGO_KSDK_ARGS += --init-args="/test/run_vsock_test.sh"
endif

ifeq ($(RELEASE_LTO), 1)
CARGO_KSDK_ARGS += --profile release-lto
KSTD_TASK_STACK_SIZE_IN_PAGES = 8
else ifeq ($(RELEASE), 1)
CARGO_KSDK_ARGS += --release
KSTD_TASK_STACK_SIZE_IN_PAGES = 8
endif

# If the BENCHMARK is set, we will run the benchmark in the kernel mode.
ifneq ($(BENCHMARK), none)
CARGO_KSDK_ARGS += --init-args="/benchmark/common/bench_runner.sh $(BENCHMARK) astros"
endif

ifeq ($(INTEL_TDX), 1)
BOOT_METHOD = grub-qcow2
BOOT_PROTOCOL = linux-efi-handover64
CARGO_KSDK_ARGS += --scheme tdx
endif

ifneq ($(SCHEME), "")
CARGO_KSDK_ARGS += --scheme $(SCHEME)
else
CARGO_KSDK_ARGS += --boot-method="$(BOOT_METHOD)"
endif

ifdef FEATURES
CARGO_KSDK_ARGS += --features="$(FEATURES)"
endif
ifeq ($(NO_DEFAULT_FEATURES), 1)
CARGO_KSDK_ARGS += --no-default-features
endif

# To test the linux-efi-handover64 boot protocol, we need to use Debian's
# GRUB release, which is installed in /usr/bin in our Docker image.
ifeq ($(BOOT_PROTOCOL), linux-efi-handover64)
CARGO_KSDK_ARGS += --grub-mkrescue=/usr/bin/grub-mkrescue
CARGO_KSDK_ARGS += --grub-boot-protocol="linux"
# FIXME: GZIP self-decompression (--encoding gzip) triggers CPU faults
CARGO_KSDK_ARGS += --encoding raw
else ifeq ($(BOOT_PROTOCOL), linux-efi-pe64)
CARGO_KSDK_ARGS += --grub-boot-protocol="linux"
CARGO_KSDK_ARGS += --encoding raw
else ifeq ($(BOOT_PROTOCOL), linux-legacy32)
CARGO_KSDK_ARGS += --linux-x86-legacy-boot
CARGO_KSDK_ARGS += --grub-boot-protocol="linux"
else
CARGO_KSDK_ARGS += --grub-boot-protocol=$(BOOT_PROTOCOL)
endif

ifeq ($(ENABLE_KVM), 1)
CARGO_KSDK_ARGS += --qemu-args="-accel kvm"
endif

# Skip GZIP to make encoding and decoding of initramfs faster
ifeq ($(INITRAMFS_SKIP_GZIP),1)
CARGO_KSDK_INITRAMFS_OPTION := --initramfs=$(realpath test/build/initramfs.cpio)
CARGO_KSDK_ARGS += $(CARGO_KSDK_INITRAMFS_OPTION)
endif

# Pass make variables to all subdirectory makes
export

# Basically, non-KSDK crates do not depend on Astros Frame and can be checked
# or tested without KSDK.
NON_KSDK_CRATES := \
	kstd/libs/align_ext \
	kstd/libs/id-alloc \
	kstd/libs/linux-bzimage/builder \
	kstd/libs/linux-bzimage/boot-params \
	kstd/libs/kstd-macros \
	kstd/libs/kstd-test \
	kernel/libs/cpio-decoder \
	kernel/libs/int-to-c-enum \
	kernel/libs/int-to-c-enum/derive \
	kernel/libs/astros-rights \
	kernel/libs/astros-rights-proc \
	kernel/libs/jhash \
	kernel/libs/keyable-arc \
	kernel/libs/typeflags \
	kernel/libs/typeflags-util \
	kernel/libs/atomic-integer-wrapper

# In contrast, KSDK crates depend on KSTD (or being `kstd` itself)
# and need to be built or tested with KSDK.
KSDK_CRATES := \
	ksdk/deps/frame-allocator \
	ksdk/deps/heap-allocator \
	ksdk/deps/test-kernel \
	kstd \
	kstd/libs/linux-bzimage/setup \
	kernel \
	kernel/comps/block \
	kernel/comps/console \
	kernel/comps/framebuffer \
	kernel/comps/input \
	kernel/comps/network \
	kernel/comps/softirq \
	kernel/comps/logger \
	kernel/comps/mlsdisk \
	kernel/comps/time \
	kernel/comps/virtio \
	kernel/comps/nvme \
	kernel/libs/astros-util \
	kernel/libs/astros-bigtcp

# KSDK dependencies
KSDK_SRC_FILES := \
	$(shell find ksdk/Cargo.toml ksdk/Cargo.lock ksdk/src -type f)

.PHONY: all
all: build

# Install or update KSDK from source
# To uninstall, do `cargo uninstall cargo-ksdk`
.PHONY: install_ksdk
install_ksdk:
	@# The `KSDK_LOCAL_DEV` environment variable is used for local development
	@# without the need to publish the changes of KSDK's self-hosted
	@# dependencies to `crates.io`.
	@KSDK_LOCAL_DEV=1 cargo install cargo-ksdk --path ksdk

# This will install and update KSDK automatically
$(CARGO_KSDK): $(KSDK_SRC_FILES)
	@$(MAKE) --no-print-directory install_ksdk

.PHONY: check_ksdk
check_ksdk:
	@cd ksdk && cargo clippy -- -D warnings

.PHONY: test_ksdk
test_ksdk:
	@cd ksdk && \
		KSDK_LOCAL_DEV=1 cargo build && \
		KSDK_LOCAL_DEV=1 cargo test

.PHONY: initramfs
initramfs:
	@$(MAKE) --no-print-directory -C test

.PHONY: build
build: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk build $(CARGO_KSDK_ARGS)

.PHONY: tools
tools:
	@cd kernel/libs/comp-sys && cargo install --path cargo-component

.PHONY: run
run: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk run $(CARGO_KSDK_ARGS)
# Check the running status of auto tests from the QEMU log
ifeq ($(AUTO_TEST), syscall)
	@tail --lines 100 qemu.log | grep -q "^.* of .* test cases passed." \
		|| (echo "Syscall test failed" && exit 1)
else ifeq ($(AUTO_TEST), test)
	@tail --lines 100 qemu.log | grep -q "^All general tests passed." \
		|| (echo "General test failed" && exit 1)
else ifeq ($(AUTO_TEST), boot)
	@tail --lines 100 qemu.log | grep -q "^Successfully booted." \
		|| (echo "Boot test failed" && exit 1)
else ifeq ($(AUTO_TEST), vsock)
	@tail --lines 100 qemu.log | grep -q "^Vsock test passed." \
		|| (echo "Vsock test failed" && exit 1)
endif

.PHONY: gdb_server
gdb_server: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk run $(CARGO_KSDK_ARGS) --gdb-server wait-client,vscode,addr=:$(GDB_TCP_PORT)

.PHONY: gdb_client
gdb_client: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk debug $(CARGO_KSDK_ARGS) --remote :$(GDB_TCP_PORT)

.PHONY: profile_server
profile_server: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk run $(CARGO_KSDK_ARGS) --gdb-server addr=:$(GDB_TCP_PORT)

.PHONY: profile_client
profile_client: initramfs $(CARGO_KSDK)
	@cd kernel && cargo ksdk profile $(CARGO_KSDK_ARGS) --remote :$(GDB_TCP_PORT) \
		--samples $(GDB_PROFILE_COUNT) --interval $(GDB_PROFILE_INTERVAL) --format $(GDB_PROFILE_FORMAT)

.PHONY: test
test:
	@for dir in $(NON_KSDK_CRATES); do \
		(cd $$dir && cargo test) || exit 1; \
	done

.PHONY: ktest
ktest: initramfs $(CARGO_KSDK)
	@# Exclude linux-bzimage-setup from ktest since it's hard to be unit tested
	@for dir in $(KSDK_CRATES); do \
		[ $$dir = "kstd/libs/linux-bzimage/setup" ] && continue; \
		echo "[make] Testing $$dir"; \
		(cd $$dir && OVMF=off cargo ksdk test $(CARGO_KSDK_INITRAMFS_OPTION)) || exit 1; \
		tail --lines 10 qemu.log | grep -q "^\\[ktest runner\\] All crates tested." \
			|| (echo "Test failed" && exit 1); \
	done

.PHONY: docs
docs: $(CARGO_KSDK)
	@for dir in $(NON_KSDK_CRATES); do \
		(cd $$dir && cargo doc --no-deps) || exit 1; \
	done
	@for dir in $(KSDK_CRATES); do \
		(cd $$dir && cargo ksdk doc --no-deps) || exit 1; \
	done
	@echo "" 						# Add a blank line
	@cd docs && mdbook build 				# Build mdBook

.PHONY: format
format:
	@./tools/format_all.sh
	@$(MAKE) --no-print-directory -C test format

.PHONY: check
check: initramfs $(CARGO_KSDK)
	@./tools/format_all.sh --check   	# Check Rust format issues
	@# Check if STD_CRATES and NKSTD_CRATES combined is the same as all workspace members
	@sed -n '/^\[workspace\]/,/^\[.*\]/{/members = \[/,/\]/p}' Cargo.toml | \
		grep -v "members = \[" | tr -d '", \]' | \
		sort > /tmp/all_crates
	@echo $(NON_KSDK_CRATES) $(KSDK_CRATES) | tr ' ' '\n' | sort > /tmp/combined_crates
	@diff -B /tmp/all_crates /tmp/combined_crates || \
		(echo "Error: The combination of STD_CRATES and NKSTD_CRATES" \
			"is not the same as all workspace members" && exit 1)
	@rm /tmp/all_crates /tmp/combined_crates
	@for dir in $(NON_KSDK_CRATES); do \
		echo "Checking $$dir"; \
		(cd $$dir && cargo clippy -- -D warnings) || exit 1; \
	done
	@for dir in $(KSDK_CRATES); do \
		echo "Checking $$dir"; \
		(cd $$dir && cargo ksdk clippy -- -- -D warnings) || exit 1; \
	done
	@$(MAKE) --no-print-directory -C test check
	@typos

.PHONY: clean
clean:
	@echo "Cleaning up Astros workspace target files"
	@cargo clean
	@echo "Cleaning up KSDK workspace target files"
	@cd ksdk && cargo clean
	@echo "Cleaning up documentation target files"
	@cd docs && mdbook clean
	@echo "Cleaning up test target files"
	@$(MAKE) --no-print-directory -C test clean
	@echo "Uninstalling KSDK"
	@rm -f $(CARGO_KSDK)
