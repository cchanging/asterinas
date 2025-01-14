# Boterinas 

## Introduction

`@botros` is a general-purpose bot designed for a wide variety of tasks in Astros. It streamlines maintenance tasks to enhance workflow efficiency. 

Commands are issued by writing comments that start with the text `@botros`. The available commands depend on which repository you are using. The main Astros repository contains a `triagebot.toml` file where you can see which features are enabled. 

Commands for GitHub issues or pull requests should be issued by writing `@botros` followed by the command anywhere in the comment. Note that `@botros` will ignore commands in Markdown code blocks, inline code spans, or blockquotes. You can enter multiple `@botros` commands in a single comment. 

For example, you can claim an issue and add a label in the same comment.
```markdown
@botros claim
@botros label C-enhancement
```

Additionally, `@botros` allows for editing comments. If you don't change the text of a command, the edit will be ignored. However, if you modify an existing command or add new ones, those commands will be processed.

Below, you'll find a comprehensive guide on how to use `@botros` effectively.

## Commands and Usage

### Workflow Management
- **`@botros rerun`**  
  Restarts the workflow of the current pull request if it has failed unexpectedly. Only the author of the pull request can use this command.

### Issue and Pull Request Management
- **`@botros claim`**  
  Assigns the issue or pull request to yourself.  
  
- **`@botros release-assignment`**  
  Removes the current assignee from an issue or pull request. This command can only be executed by the current assignee or a team member.  
  
- **`@botros assign @user`**  
  Assigns a specific user to the issue or pull request. Only team members have permission to assign other users.  

### Label Management
- **`@botros label <label>`**  
  Adds a label to the issue or pull request.  
  *Example:* `@botros label C-enhancement C-rfc`
  
- **`@botros label -<label>`**  
  Removes a label from the issue or pull request.  
  *Example:* `@botros label -C-enhancement -C-bug`

### Status Indicators
- **`@botros author`**  
  Indicates that a pull request is waiting on the author. It assigns the `S-waiting-on-author` label and removes both `S-waiting-on-review` and `S-blocked`, if present.  
  
- **`@botros blocked`**  
  Marks a pull request as blocked on something.  
  
- **`@botros ready`**  
  Indicates that a pull request is ready for review. This command can also be invoked with the aliases `@botros review` or `@botros reviewer`.  

## Notes
- Only team members can assign users or remove assignments.
- Labels are crucial for organizing issues and pull requests, so ensure they are used consistently and accurately.
- For any issues or questions regarding `@botros`, please reach out to the team for support.
