<!--
  generated from agentide v1
  model digest 6533259c5264c4a5da367c055615054401fe7a49a71a2a87a3f87bfa2a3dfbe4
  contract digest f1c2495df22f205934254ade5d845a2b65f2cbf16c4e9dfa1e863a24ed2d3b17
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — agentide v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

184 capabilities: **125 generated**, **52 obligations**, **7 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `agentide.coding.AgentId` |
| domain type | `agentide.coding.ExpectedDigest` |
| domain type | `agentide.coding.OperationId` |
| domain type | `agentide.coding.Path` |
| domain type | `agentide.coding.Pattern` |
| domain type | `agentide.coding.ProcessId` |
| domain type | `agentide.coding.ProcessProfile` |
| domain type | `agentide.coding.Signal` |
| domain type | `agentide.coding.TerminalId` |
| domain type | `agentide.coding.TerminalProfile` |
| domain type | `agentide.coding.VerificationLevel` |
| domain type | `agentide.coordination.ApprovalCheckpoint.State` |
| domain type | `agentide.coordination.AuthorityGrant.State` |
| domain type | `agentide.coordination.CheckpointId` |
| domain type | `agentide.coordination.ContextKind` |
| domain type | `agentide.coordination.ContextPin.State` |
| domain type | `agentide.coordination.ContextPinId` |
| domain type | `agentide.coordination.Digest` |
| domain type | `agentide.coordination.GrantId` |
| domain type | `agentide.coordination.GrantRisk` |
| domain type | `agentide.coordination.IntentRef` |
| domain type | `agentide.coordination.OpaqueRef` |
| domain type | `agentide.coordination.PathPrefix` |
| domain type | `agentide.coordination.SubjectRef` |
| domain type | `agentide.session.CodingSession.State` |
| domain type | `agentide.session.Cursor` |
| domain type | `agentide.session.EventLimit` |
| domain type | `agentide.session.ManifestDigest` |
| domain type | `agentide.session.ProjectId` |
| domain type | `agentide.session.RequestId` |
| domain type | `agentide.session.SessionId` |
| domain type | `agentide.session.SessionScopes` |
| domain type | `agentide.session.SourceRevision` |
| domain type | `agentide.session.WorkspaceRoot` |
| domain type | `agentide.session.WorkspaceSessionId` |
| domain type | `agentide.surface.PaneId` |
| domain type | `agentide.surface.PaneKind` |
| domain type | `agentide.surface.PaneSnapshot` |
| domain type | `agentide.surface.SplitDirection` |
| domain type | `agentide.surface.Workbench.State` |
| entity lifecycle | `agentide.coordination.ApprovalCheckpoint` |
| entity lifecycle | `agentide.coordination.AuthorityGrant` |
| entity lifecycle | `agentide.coordination.ContextPin` |
| entity lifecycle | `agentide.session.CodingSession` |
| entity lifecycle | `agentide.surface.Workbench` |
| command contract | `agentide.coding.ApplyDeployment` |
| command contract | `agentide.coding.CancelProcess` |
| command contract | `agentide.coding.CreateCode` |
| command contract | `agentide.coding.CreateWorktree` |
| command contract | `agentide.coding.CutRelease` |
| command contract | `agentide.coding.DelegateAgent` |
| command contract | `agentide.coding.DeleteCode` |
| command contract | `agentide.coding.EditCode` |
| command contract | `agentide.coding.FinishWorktree` |
| command contract | `agentide.coding.InputProcess` |
| command contract | `agentide.coding.ListTerminals` |
| command contract | `agentide.coding.MessageAgent` |
| command contract | `agentide.coding.ObserveAgents` |
| command contract | `agentide.coding.ObserveChanges` |
| command contract | `agentide.coding.ObserveDeployment` |
| command contract | `agentide.coding.ObserveProcess` |
| command contract | `agentide.coding.ObserveWorktree` |
| command contract | `agentide.coding.OpenInteractiveTerminal` |
| command contract | `agentide.coding.PublishCode` |
| command contract | `agentide.coding.ReadCode` |
| command contract | `agentide.coding.RecordEvidence` |
| command contract | `agentide.coding.RenameCode` |
| command contract | `agentide.coding.SearchCode` |
| command contract | `agentide.coding.StartProcess` |
| command contract | `agentide.coding.TerminateTerminal` |
| command contract | `agentide.coding.VerifyCode` |
| command contract | `agentide.coding.WaitAgent` |
| command contract | `agentide.coding.WaitProcess` |
| command contract | `agentide.coordination.ApproveCheckpoint` |
| command contract | `agentide.coordination.CreateGrant` |
| command contract | `agentide.coordination.DenyCheckpoint` |
| command contract | `agentide.coordination.PinContext` |
| command contract | `agentide.coordination.RecordApprovalCheckpoint` |
| command contract | `agentide.coordination.RemoveContextPin` |
| command contract | `agentide.coordination.RevokeGrant` |
| command contract | `agentide.session.CloseSession` |
| command contract | `agentide.session.ReadEvents` |
| command contract | `agentide.session.SnapshotSession` |
| command contract | `agentide.session.StartSession` |
| command contract | `agentide.surface.CloseFile` |
| command contract | `agentide.surface.ClosePane` |
| command contract | `agentide.surface.FocusPane` |
| command contract | `agentide.surface.MoveCursor` |
| command contract | `agentide.surface.OpenFile` |
| command contract | `agentide.surface.OpenPane` |
| command contract | `agentide.surface.ShowDiff` |
| command contract | `agentide.surface.SnapshotSurface` |
| event type | `agentide.coding.IntentCompleted` |
| event type | `agentide.coding.IntentRefused` |
| event type | `agentide.coordination.ApprovalCheckpointApproved` |
| event type | `agentide.coordination.ApprovalCheckpointDenied` |
| event type | `agentide.coordination.ApprovalCheckpointRecorded` |
| event type | `agentide.coordination.ContextPinRemoved` |
| event type | `agentide.coordination.ContextPinned` |
| event type | `agentide.coordination.GrantCreated` |
| event type | `agentide.coordination.GrantRevoked` |
| event type | `agentide.session.SessionClosed` |
| event type | `agentide.session.SessionObserved` |
| event type | `agentide.session.SessionStarted` |
| event type | `agentide.surface.CursorMoved` |
| event type | `agentide.surface.DiffShown` |
| event type | `agentide.surface.FileClosed` |
| event type | `agentide.surface.FileOpened` |
| event type | `agentide.surface.PaneClosed` |
| event type | `agentide.surface.PaneFocused` |
| event type | `agentide.surface.PaneOpened` |
| event type | `agentide.surface.SurfaceObserved` |
| error type | `agentide.coding.IntentFailure` |
| error type | `agentide.coordination.CoordinationRefusal` |
| error type | `agentide.coordination.CoordinationStateConflict` |
| error type | `agentide.session.SessionRefusal` |
| error type | `agentide.session.SessionStateConflict` |
| error type | `agentide.surface.SurfaceFailure` |
| view type | `agentide.coordination.ApprovalCheckpointSnapshot` |
| view type | `agentide.coordination.ContextPinSnapshot` |
| view type | `agentide.coordination.GrantSnapshot` |
| view type | `agentide.session.SessionSnapshot` |
| view type | `agentide.surface.WorkbenchSnapshot` |
| component port | `agentide-engine` |
| component transport | `agentide-engine` |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `agentide.coding.ApplyDeployment` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ApplyDeployment` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.CancelProcess` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.CancelProcess` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.CreateCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.CreateCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.CreateWorktree` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.CreateWorktree` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.CutRelease` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.CutRelease` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.DelegateAgent` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.DelegateAgent` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.DeleteCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.DeleteCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.EditCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.EditCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.FinishWorktree` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.FinishWorktree` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.InputProcess` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.InputProcess` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ListTerminals` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ListTerminals` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.MessageAgent` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.MessageAgent` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ObserveAgents` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ObserveAgents` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ObserveChanges` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ObserveChanges` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ObserveDeployment` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ObserveDeployment` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ObserveProcess` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ObserveProcess` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ObserveWorktree` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ObserveWorktree` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.OpenInteractiveTerminal` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.OpenInteractiveTerminal` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.PublishCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.PublishCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.ReadCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.ReadCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.RecordEvidence` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.RecordEvidence` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.RenameCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.RenameCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.SearchCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.SearchCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.StartProcess` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.StartProcess` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.TerminateTerminal` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.TerminateTerminal` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.VerifyCode` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.VerifyCode` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.WaitAgent` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.WaitAgent` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coding.WaitProcess` | decided outside the system: the selected implementation cannot admit or complete the intent | given `agentide.coding.WaitProcess` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.coding.IntentCompleted`; `refused` externally decided (the selected implementation cannot admit or complete the intent), error `agentide.coding.IntentFailure` |
| command behaviour | `agentide.coordination.ApproveCheckpoint` | decided outside the system: durable state is unavailable | given `agentide.coordination.ApproveCheckpoint` input, decide and enact exactly one outcome — `approved` otherwise, takes `approve` of `agentide.coordination.ApprovalCheckpoint`, emits `agentide.coordination.ApprovalCheckpointApproved`; `wrong-state` from a state no declared move starts in, error `agentide.coordination.CoordinationStateConflict`; `refused` externally decided (durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.CreateGrant` | decided outside the system: authority or durable state is unavailable | given `agentide.coordination.CreateGrant` input, decide and enact exactly one outcome — `created` otherwise, creates `agentide.coordination.AuthorityGrant`, emits `agentide.coordination.GrantCreated`; `refused` externally decided (authority or durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.DenyCheckpoint` | decided outside the system: durable state is unavailable | given `agentide.coordination.DenyCheckpoint` input, decide and enact exactly one outcome — `denied` otherwise, takes `deny` of `agentide.coordination.ApprovalCheckpoint`, emits `agentide.coordination.ApprovalCheckpointDenied`; `wrong-state` from a state no declared move starts in, error `agentide.coordination.CoordinationStateConflict`; `refused` externally decided (durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.PinContext` | decided outside the system: the reference cannot be admitted or durable state is unavailable | given `agentide.coordination.PinContext` input, decide and enact exactly one outcome — `pinned` otherwise, creates `agentide.coordination.ContextPin`, emits `agentide.coordination.ContextPinned`; `refused` externally decided (the reference cannot be admitted or durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.RecordApprovalCheckpoint` | decided outside the system: durable state is unavailable | given `agentide.coordination.RecordApprovalCheckpoint` input, decide and enact exactly one outcome — `recorded` otherwise, creates `agentide.coordination.ApprovalCheckpoint`, emits `agentide.coordination.ApprovalCheckpointRecorded`; `refused` externally decided (durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.RemoveContextPin` | decided outside the system: durable state is unavailable | given `agentide.coordination.RemoveContextPin` input, decide and enact exactly one outcome — `removed` otherwise, takes `remove` of `agentide.coordination.ContextPin`, emits `agentide.coordination.ContextPinRemoved`; `wrong-state` from a state no declared move starts in, error `agentide.coordination.CoordinationStateConflict`; `refused` externally decided (durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.coordination.RevokeGrant` | decided outside the system: durable state is unavailable | given `agentide.coordination.RevokeGrant` input, decide and enact exactly one outcome — `revoked` otherwise, takes `revoke` of `agentide.coordination.AuthorityGrant`, emits `agentide.coordination.GrantRevoked`; `wrong-state` from a state no declared move starts in, error `agentide.coordination.CoordinationStateConflict`; `refused` externally decided (durable state is unavailable), error `agentide.coordination.CoordinationRefusal` |
| command behaviour | `agentide.session.CloseSession` | decided outside the system: durable state cannot be updated | given `agentide.session.CloseSession` input, decide and enact exactly one outcome — `closed` otherwise, takes `close` of `agentide.session.CodingSession`, emits `agentide.session.SessionClosed`; `wrong-state` from a state no declared move starts in, error `agentide.session.SessionStateConflict`; `refused` externally decided (durable state cannot be updated), error `agentide.session.SessionRefusal` |
| command behaviour | `agentide.session.ReadEvents` | decided outside the system: the requested event window cannot be represented | given `agentide.session.ReadEvents` input, decide and enact exactly one outcome — `observed` otherwise, emits `agentide.session.SessionObserved`; `refused` externally decided (the requested event window cannot be represented), error `agentide.session.SessionRefusal` |
| command behaviour | `agentide.session.SnapshotSession` | decided outside the system: the durable session cannot be read exactly | given `agentide.session.SnapshotSession` input, decide and enact exactly one outcome — `observed` otherwise, emits `agentide.session.SessionObserved`; `refused` externally decided (the durable session cannot be read exactly), error `agentide.session.SessionRefusal` |
| command behaviour | `agentide.session.StartSession` | decided outside the system: the workspace or standalone binding is unavailable | given `agentide.session.StartSession` input, decide and enact exactly one outcome — `started` otherwise, creates `agentide.session.CodingSession`, emits `agentide.session.SessionStarted`; `refused` externally decided (the workspace or standalone binding is unavailable), error `agentide.session.SessionRefusal` |
| command behaviour | `agentide.surface.CloseFile` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.CloseFile` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.ClosePane` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.ClosePane` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.FocusPane` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.FocusPane` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.MoveCursor` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.MoveCursor` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.OpenFile` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.OpenFile` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.OpenPane` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.OpenPane` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.ShowDiff` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.ShowDiff` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| command behaviour | `agentide.surface.SnapshotSurface` | decided outside the system: the requested surface state is unavailable | given `agentide.surface.SnapshotSurface` input, decide and enact exactly one outcome — `completed` otherwise, emits `agentide.surface.SurfaceObserved`; `refused` externally decided (the requested surface state is unavailable), error `agentide.surface.SurfaceFailure` |
| view query | `agentide.coordination.ApprovalCheckpointSnapshot` | how the projection is kept current is a storage decision | a query answering `agentide.coordination.ApprovalCheckpointSnapshot` with rows projected from `agentide.coordination.ApprovalCheckpoint` at `read_your_writes` consistency |
| view query | `agentide.coordination.ContextPinSnapshot` | how the projection is kept current is a storage decision | a query answering `agentide.coordination.ContextPinSnapshot` with rows projected from `agentide.coordination.ContextPin` at `read_your_writes` consistency |
| view query | `agentide.coordination.GrantSnapshot` | how the projection is kept current is a storage decision | a query answering `agentide.coordination.GrantSnapshot` with rows projected from `agentide.coordination.AuthorityGrant` at `read_your_writes` consistency |
| view query | `agentide.session.SessionSnapshot` | how the projection is kept current is a storage decision | a query answering `agentide.session.SessionSnapshot` with rows projected from `agentide.session.CodingSession` at `read_your_writes` consistency |
| view query | `agentide.surface.WorkbenchSnapshot` | how the projection is kept current is a storage decision | a query answering `agentide.surface.WorkbenchSnapshot` with rows projected from `agentide.surface.Workbench` at `read_your_writes` consistency |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `agentide.coding.CodingAgent` | planning | may invoke `agentide.coding.ApplyDeployment`, `agentide.coding.CancelProcess`, `agentide.coding.CreateCode`, `agentide.coding.CreateWorktree`, `agentide.coding.CutRelease`, `agentide.coding.DelegateAgent`, `agentide.coding.DeleteCode`, `agentide.coding.EditCode`, `agentide.coding.FinishWorktree`, `agentide.coding.InputProcess`, `agentide.coding.ListTerminals`, `agentide.coding.MessageAgent`, `agentide.coding.ObserveAgents`, `agentide.coding.ObserveChanges`, `agentide.coding.ObserveDeployment`, `agentide.coding.ObserveProcess`, `agentide.coding.ObserveWorktree`, `agentide.coding.OpenInteractiveTerminal`, `agentide.coding.PublishCode`, `agentide.coding.ReadCode`, `agentide.coding.RecordEvidence`, `agentide.coding.RenameCode`, `agentide.coding.SearchCode`, `agentide.coding.StartProcess`, `agentide.coding.TerminateTerminal`, `agentide.coding.VerifyCode`, `agentide.coding.WaitAgent`, `agentide.coding.WaitProcess`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.coordination.SessionExecutor` | planning | may invoke `agentide.coordination.PinContext`, `agentide.coordination.RemoveContextPin`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.coordination.SessionOwner` | planning | may invoke `agentide.coordination.ApproveCheckpoint`, `agentide.coordination.CreateGrant`, `agentide.coordination.DenyCheckpoint`, `agentide.coordination.PinContext`, `agentide.coordination.RemoveContextPin`, `agentide.coordination.RevokeGrant`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.coordination.TaskAuthority` | planning | may invoke `agentide.coordination.RecordApprovalCheckpoint`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.session.CodingAgent` | planning | may invoke `agentide.session.ReadEvents`, `agentide.session.SnapshotSession`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.session.Operator` | planning | may invoke `agentide.session.CloseSession`, `agentide.session.StartSession`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `agentide.surface.CodingAgent` | planning | may invoke `agentide.surface.CloseFile`, `agentide.surface.ClosePane`, `agentide.surface.FocusPane`, `agentide.surface.MoveCursor`, `agentide.surface.OpenFile`, `agentide.surface.OpenPane`, `agentide.surface.ShowDiff`, `agentide.surface.SnapshotSurface`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
