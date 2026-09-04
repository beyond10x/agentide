<!--
generated from agentide v1
model digest 509495079a366d767a747dbfcd22e419c28040b7ff32d15a1f284393168d16ab
contract digest fe5c3496c236e0bc543c988d0cafdbcbd093b23a8e0e4cff4dd33f5733a5b07b
do not edit: regenerate with `ess generate`
-->

# Coding session

Semantic observation, change, execution, collaboration, evidence, and delivery intents.

`agentide.coding` is one of agentide's bounded contexts. [Back to the index](../index.md).

## Types

### `AgentId`

`agentide.coding.AgentId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `ExpectedDigest`

`agentide.coding.ExpectedDigest` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `OperationId`

`agentide.coding.OperationId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `Path`

`agentide.coding.Path` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `Pattern`

`agentide.coding.Pattern` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `ProcessId`

`agentide.coding.ProcessId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `ProcessProfile`

`agentide.coding.ProcessProfile` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `Signal`

`agentide.coding.Signal` is one of `Interrupt`, `Terminate` and `Kill`.

### `TerminalId`

`agentide.coding.TerminalId` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `TerminalProfile`

`agentide.coding.TerminalProfile` wraps `String` and is not interchangeable with one: the whole value of naming it separately is the crossings the model then refuses.

### `VerificationLevel`

`agentide.coding.VerificationLevel` is one of `Focused`, `Full` and `Release`.

## Commands

### `ApplyDeployment`

`agentide.coding.ApplyDeployment`, shown to a person as "Apply deployment" and called `deployment-apply` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `environment` — `String`
- `revision` — `agentide.coding.ExpectedDigest`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `CancelProcess`

`agentide.coding.CancelProcess`, shown to a person as "Cancel process" and called `process-cancel` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `process_id` — `agentide.coding.ProcessId`
- `signal` — `agentide.coding.Signal`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `CreateCode`

`agentide.coding.CreateCode`, shown to a person as "Create code" and called `code-create` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `path` — `agentide.coding.Path`
- `content` — `String`
- `expected_absent` — `Boolean`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `CreateWorktree`

`agentide.coding.CreateWorktree`, shown to a person as "Create worktree" and called `worktree-create` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `purpose` — `String`
- `base` — `Optional<String>`, which may be absent

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `CutRelease`

`agentide.coding.CutRelease`, shown to a person as "Cut release" and called `release-cut` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `version` — `String`
- `expected_head` — `agentide.coding.ExpectedDigest`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `DelegateAgent`

`agentide.coding.DelegateAgent`, shown to a person as "Delegate to agent" and called `agent-delegate` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `task` — `String`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `DeleteCode`

`agentide.coding.DeleteCode`, shown to a person as "Delete code" and called `code-delete` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `path` — `agentide.coding.Path`
- `expected_sha256` — `agentide.coding.ExpectedDigest`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `EditCode`

`agentide.coding.EditCode`, shown to a person as "Edit code" and called `code-edit` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `path` — `agentide.coding.Path`
- `content` — `String`
- `expected_sha256` — `Optional<agentide.coding.ExpectedDigest>`, which may be absent

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `FinishWorktree`

`agentide.coding.FinishWorktree`, shown to a person as "Finish worktree" and called `worktree-finish` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `InputProcess`

`agentide.coding.InputProcess`, shown to a person as "Write process input" and called `process-input` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `process_id` — `agentide.coding.ProcessId`
- `content` — `String`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ListTerminals`

`agentide.coding.ListTerminals`, shown to a person as "List terminals" and called `terminal-list` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `MessageAgent`

`agentide.coding.MessageAgent`, shown to a person as "Message agent" and called `agent-message` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `agent_id` — `agentide.coding.AgentId`
- `message` — `String`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ObserveAgents`

`agentide.coding.ObserveAgents`, shown to a person as "Observe agents" and called `agent-observe` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ObserveChanges`

`agentide.coding.ObserveChanges`, shown to a person as "Observe code changes" and called `code-changes` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ObserveDeployment`

`agentide.coding.ObserveDeployment`, shown to a person as "Observe deployment" and called `deployment-observe` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `environment` — `String`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ObserveProcess`

`agentide.coding.ObserveProcess`, shown to a person as "Observe process" and called `process-observe` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `process_id` — `agentide.coding.ProcessId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ObserveWorktree`

`agentide.coding.ObserveWorktree`, shown to a person as "Observe worktree" and called `worktree-status` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `OpenInteractiveTerminal`

`agentide.coding.OpenInteractiveTerminal`, shown to a person as "Open interactive terminal" and called `interactive-terminal` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `profile` — `agentide.coding.TerminalProfile`
- `working_directory` — `agentide.coding.Path`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `PublishCode`

`agentide.coding.PublishCode`, shown to a person as "Publish code" and called `code-publish` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `expected_head` — `agentide.coding.ExpectedDigest`
- `summary` — `String`
- `evidence` — `List<String>`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `ReadCode`

`agentide.coding.ReadCode`, shown to a person as "Read code" and called `code-read` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `path` — `agentide.coding.Path`
- `offset` — `Integer`
- `limit_bytes` — `Integer`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `RecordEvidence`

`agentide.coding.RecordEvidence`, shown to a person as "Record evidence" and called `evidence-record` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `subject` — `String`
- `kind` — `String`
- `source` — `String`
- `reference` — `Optional<String>`, which may be absent

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `RenameCode`

`agentide.coding.RenameCode`, shown to a person as "Rename code" and called `code-rename` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `path` — `agentide.coding.Path`
- `destination` — `agentide.coding.Path`
- `expected_sha256` — `agentide.coding.ExpectedDigest`
- `destination_expected_absent` — `Boolean`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `SearchCode`

`agentide.coding.SearchCode`, shown to a person as "Search code" and called `code-search` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `pattern` — `agentide.coding.Pattern`
- `paths` — `List<agentide.coding.Path>`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `StartProcess`

`agentide.coding.StartProcess`, shown to a person as "Start process" and called `process-start` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `profile` — `agentide.coding.ProcessProfile`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `TerminateTerminal`

`agentide.coding.TerminateTerminal`, shown to a person as "Terminate terminal" and called `terminal-terminate` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `terminal_id` — `agentide.coding.TerminalId`
- `signal` — `agentide.coding.Signal`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `VerifyCode`

`agentide.coding.VerifyCode`, shown to a person as "Verify code" and called `code-verify` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `operation_id` — `agentide.coding.OperationId`
- `level` — `agentide.coding.VerificationLevel`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `WaitAgent`

`agentide.coding.WaitAgent`, shown to a person as "Wait for agents" and called `agent-wait` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `agent_ids` — `List<agentide.coding.AgentId>`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

### `WaitProcess`

`agentide.coding.WaitProcess`, shown to a person as "Wait for process" and called `process-wait` on the wire.

It takes:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `process_id` — `agentide.coding.ProcessId`
- `timeout_ms` — `Integer`

It has two outcomes.

**`completed`** — The default branch, taken when no other outcome's condition matched. No entity in this specification changes. It emits `agentide.coding.IntentCompleted`. A test reaches it by constructing an input that satisfies no other outcome's condition.

**`refused`** — Decided outside the input: the selected implementation cannot admit or complete the intent. No predicate over the input reaches this branch, and saying `when: false` instead would have claimed it is unreachable, which is a different and false statement. No entity in this specification changes. It reports `agentide.coding.IntentFailure`, carrying `code`, `message` and `retryable`. It emits nothing. A test reaches it by injecting the declared fault, because no input can.

## Events

### `IntentCompleted`

`agentide.coding.IntentCompleted`.

It carries:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`

Emitted by `agentide.coding.ApplyDeployment` on its `completed` outcome.

Emitted by `agentide.coding.CancelProcess` on its `completed` outcome.

Emitted by `agentide.coding.CreateCode` on its `completed` outcome.

Emitted by `agentide.coding.CreateWorktree` on its `completed` outcome.

Emitted by `agentide.coding.CutRelease` on its `completed` outcome.

Emitted by `agentide.coding.DelegateAgent` on its `completed` outcome.

Emitted by `agentide.coding.DeleteCode` on its `completed` outcome.

Emitted by `agentide.coding.EditCode` on its `completed` outcome.

Emitted by `agentide.coding.FinishWorktree` on its `completed` outcome.

Emitted by `agentide.coding.InputProcess` on its `completed` outcome.

Emitted by `agentide.coding.ListTerminals` on its `completed` outcome.

Emitted by `agentide.coding.MessageAgent` on its `completed` outcome.

Emitted by `agentide.coding.ObserveAgents` on its `completed` outcome.

Emitted by `agentide.coding.ObserveChanges` on its `completed` outcome.

Emitted by `agentide.coding.ObserveDeployment` on its `completed` outcome.

Emitted by `agentide.coding.ObserveProcess` on its `completed` outcome.

Emitted by `agentide.coding.ObserveWorktree` on its `completed` outcome.

Emitted by `agentide.coding.OpenInteractiveTerminal` on its `completed` outcome.

Emitted by `agentide.coding.PublishCode` on its `completed` outcome.

Emitted by `agentide.coding.ReadCode` on its `completed` outcome.

Emitted by `agentide.coding.RecordEvidence` on its `completed` outcome.

Emitted by `agentide.coding.RenameCode` on its `completed` outcome.

Emitted by `agentide.coding.SearchCode` on its `completed` outcome.

Emitted by `agentide.coding.StartProcess` on its `completed` outcome.

Emitted by `agentide.coding.TerminateTerminal` on its `completed` outcome.

Emitted by `agentide.coding.VerifyCode` on its `completed` outcome.

Emitted by `agentide.coding.WaitAgent` on its `completed` outcome.

Emitted by `agentide.coding.WaitProcess` on its `completed` outcome.

Nothing in this system reacts to it.

### `IntentRefused`

`agentide.coding.IntentRefused`.

It carries:

- `session_id` — `agentide.session.SessionId`
- `request_id` — `agentide.session.RequestId`
- `code` — `String`

No command in this system emits it, so something outside the specification does.

Nothing in this system reacts to it.

## Errors

### `IntentFailure`

The intent was refused or failed without claiming the requested effect happened.

It carries:

- `code` — `String`
- `message` — `String`
- `retryable` — `Boolean`

Reported by `agentide.coding.ApplyDeployment` on its `refused` outcome.

Reported by `agentide.coding.CancelProcess` on its `refused` outcome.

Reported by `agentide.coding.CreateCode` on its `refused` outcome.

Reported by `agentide.coding.CreateWorktree` on its `refused` outcome.

Reported by `agentide.coding.CutRelease` on its `refused` outcome.

Reported by `agentide.coding.DelegateAgent` on its `refused` outcome.

Reported by `agentide.coding.DeleteCode` on its `refused` outcome.

Reported by `agentide.coding.EditCode` on its `refused` outcome.

Reported by `agentide.coding.FinishWorktree` on its `refused` outcome.

Reported by `agentide.coding.InputProcess` on its `refused` outcome.

Reported by `agentide.coding.ListTerminals` on its `refused` outcome.

Reported by `agentide.coding.MessageAgent` on its `refused` outcome.

Reported by `agentide.coding.ObserveAgents` on its `refused` outcome.

Reported by `agentide.coding.ObserveChanges` on its `refused` outcome.

Reported by `agentide.coding.ObserveDeployment` on its `refused` outcome.

Reported by `agentide.coding.ObserveProcess` on its `refused` outcome.

Reported by `agentide.coding.ObserveWorktree` on its `refused` outcome.

Reported by `agentide.coding.OpenInteractiveTerminal` on its `refused` outcome.

Reported by `agentide.coding.PublishCode` on its `refused` outcome.

Reported by `agentide.coding.ReadCode` on its `refused` outcome.

Reported by `agentide.coding.RecordEvidence` on its `refused` outcome.

Reported by `agentide.coding.RenameCode` on its `refused` outcome.

Reported by `agentide.coding.SearchCode` on its `refused` outcome.

Reported by `agentide.coding.StartProcess` on its `refused` outcome.

Reported by `agentide.coding.TerminateTerminal` on its `refused` outcome.

Reported by `agentide.coding.VerifyCode` on its `refused` outcome.

Reported by `agentide.coding.WaitAgent` on its `refused` outcome.

Reported by `agentide.coding.WaitProcess` on its `refused` outcome.

## Actors

An actor is who may ask this context for something. Every grant below points at a command this specification declares — a grant is a resolved reference, so "may invoke" something nobody wrote is not a permission this model can express, and an authorisation that authorises nothing cannot ship quietly.

### `CodingAgent`

`agentide.coding.CodingAgent`.

It may invoke [`ApplyDeployment`](#applydeployment), [`CancelProcess`](#cancelprocess), [`CreateCode`](#createcode), [`CreateWorktree`](#createworktree), [`CutRelease`](#cutrelease), [`DelegateAgent`](#delegateagent), [`DeleteCode`](#deletecode), [`EditCode`](#editcode), [`FinishWorktree`](#finishworktree), [`InputProcess`](#inputprocess), [`ListTerminals`](#listterminals), [`MessageAgent`](#messageagent), [`ObserveAgents`](#observeagents), [`ObserveChanges`](#observechanges), [`ObserveDeployment`](#observedeployment), [`ObserveProcess`](#observeprocess), [`ObserveWorktree`](#observeworktree), [`OpenInteractiveTerminal`](#openinteractiveterminal), [`PublishCode`](#publishcode), [`ReadCode`](#readcode), [`RecordEvidence`](#recordevidence), [`RenameCode`](#renamecode), [`SearchCode`](#searchcode), [`StartProcess`](#startprocess), [`TerminateTerminal`](#terminateterminal), [`VerifyCode`](#verifycode), [`WaitAgent`](#waitagent) and [`WaitProcess`](#waitprocess).


---

Generated from agentide v1 · model digest `509495079a366d767a747dbfcd22e419c28040b7ff32d15a1f284393168d16ab` · contract digest `fe5c3496c236e0bc543c988d0cafdbcbd093b23a8e0e4cff4dd33f5733a5b07b`. Do not edit this file; change the specification and regenerate it with `ess generate`.
