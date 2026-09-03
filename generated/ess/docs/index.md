<!--
generated from agentide v1
model digest 360a1e3f4110754181c30d72530c91ae4344a5f5d4c3aff968bda22d67ca12f3
contract digest 90e1687e023e28f77eca4934db634944dcaf13a22a2075c642452ec5a97822b4
do not edit: regenerate with `ess generate`
-->

# agentide v1

A coding-session interface in which agents invoke stable semantic intents while deployments bind those intents to guarded implementations and humans observe the same durable event projection.

## The system as a graph

```mermaid
flowchart TB
    subgraph who["who may ask"]
        who0["agentide.coding.CodingAgent"]
        who1["agentide.coordination.SessionExecutor"]
        who2["agentide.coordination.SessionOwner"]
        who3["agentide.coordination.TaskAuthority"]
        who4["agentide.session.CodingAgent"]
        who5["agentide.session.Operator"]
        who6["agentide.surface.CodingAgent"]
    end
    subgraph unit0["agentide-engine"]
        cmd0["agentide.coding.ApplyDeployment"]
        cmd1["agentide.coding.CancelProcess"]
        cmd2["agentide.coding.CreateCode"]
        cmd3["agentide.coding.CreateWorktree"]
        cmd4["agentide.coding.CutRelease"]
        cmd5["agentide.coding.DelegateAgent"]
        cmd6["agentide.coding.DeleteCode"]
        cmd7["agentide.coding.EditCode"]
        cmd8["agentide.coding.FinishWorktree"]
        cmd9["agentide.coding.InputProcess"]
        cmd10["agentide.coding.ListTerminals"]
        cmd11["agentide.coding.MessageAgent"]
        cmd12["agentide.coding.ObserveAgents"]
        cmd13["agentide.coding.ObserveChanges"]
        cmd14["agentide.coding.ObserveDeployment"]
        cmd15["agentide.coding.ObserveProcess"]
        cmd16["agentide.coding.ObserveWorktree"]
        cmd17["agentide.coding.OpenInteractiveTerminal"]
        cmd18["agentide.coding.PublishCode"]
        cmd19["agentide.coding.ReadCode"]
        cmd20["agentide.coding.RecordEvidence"]
        cmd21["agentide.coding.RenameCode"]
        cmd22["agentide.coding.SearchCode"]
        cmd23["agentide.coding.StartProcess"]
        cmd24["agentide.coding.TerminateTerminal"]
        cmd25["agentide.coding.VerifyCode"]
        cmd26["agentide.coding.WaitAgent"]
        cmd27["agentide.coding.WaitProcess"]
        cmd28["agentide.coordination.ApproveCheckpoint"]
        cmd29["agentide.coordination.CreateGrant"]
        cmd30["agentide.coordination.DenyCheckpoint"]
        cmd31["agentide.coordination.PinContext"]
        cmd32["agentide.coordination.RecordApprovalCheckpoint"]
        cmd33["agentide.coordination.RemoveContextPin"]
        cmd34["agentide.coordination.RevokeGrant"]
        cmd35["agentide.session.CloseSession"]
        cmd36["agentide.session.EnsureHostedSession"]
        cmd37["agentide.session.ReadEvents"]
        cmd38["agentide.session.SnapshotSession"]
        cmd39["agentide.session.StartSession"]
        cmd40["agentide.surface.CloseFile"]
        cmd41["agentide.surface.ClosePane"]
        cmd42["agentide.surface.FocusPane"]
        cmd43["agentide.surface.MoveCursor"]
        cmd44["agentide.surface.OpenFile"]
        cmd45["agentide.surface.OpenPane"]
        cmd46["agentide.surface.ShowDiff"]
        cmd47["agentide.surface.SnapshotSurface"]
        evt0["agentide.coding.IntentCompleted"]
        evt1["agentide.coding.IntentRefused"]
        evt2["agentide.coordination.ApprovalCheckpointApproved"]
        evt3["agentide.coordination.ApprovalCheckpointDenied"]
        evt4["agentide.coordination.ApprovalCheckpointRecorded"]
        evt5["agentide.coordination.ContextPinRemoved"]
        evt6["agentide.coordination.ContextPinned"]
        evt7["agentide.coordination.GrantCreated"]
        evt8["agentide.coordination.GrantRevoked"]
        evt9["agentide.session.SessionClosed"]
        evt10["agentide.session.SessionObserved"]
        evt11["agentide.session.SessionStarted"]
        evt12["agentide.surface.CursorMoved"]
        evt13["agentide.surface.DiffShown"]
        evt14["agentide.surface.FileClosed"]
        evt15["agentide.surface.FileOpened"]
        evt16["agentide.surface.PaneClosed"]
        evt17["agentide.surface.PaneFocused"]
        evt18["agentide.surface.PaneOpened"]
        evt19["agentide.surface.SurfaceObserved"]
    end
    who0 -->|"may invoke"| cmd0
    who0 -->|"may invoke"| cmd1
    who0 -->|"may invoke"| cmd2
    who0 -->|"may invoke"| cmd3
    who0 -->|"may invoke"| cmd4
    who0 -->|"may invoke"| cmd5
    who0 -->|"may invoke"| cmd6
    who0 -->|"may invoke"| cmd7
    who0 -->|"may invoke"| cmd8
    who0 -->|"may invoke"| cmd9
    who0 -->|"may invoke"| cmd10
    who0 -->|"may invoke"| cmd11
    who0 -->|"may invoke"| cmd12
    who0 -->|"may invoke"| cmd13
    who0 -->|"may invoke"| cmd14
    who0 -->|"may invoke"| cmd15
    who0 -->|"may invoke"| cmd16
    who0 -->|"may invoke"| cmd17
    who0 -->|"may invoke"| cmd18
    who0 -->|"may invoke"| cmd19
    who0 -->|"may invoke"| cmd20
    who0 -->|"may invoke"| cmd21
    who0 -->|"may invoke"| cmd22
    who0 -->|"may invoke"| cmd23
    who0 -->|"may invoke"| cmd24
    who0 -->|"may invoke"| cmd25
    who0 -->|"may invoke"| cmd26
    who0 -->|"may invoke"| cmd27
    who1 -->|"may invoke"| cmd31
    who1 -->|"may invoke"| cmd33
    who2 -->|"may invoke"| cmd28
    who2 -->|"may invoke"| cmd29
    who2 -->|"may invoke"| cmd30
    who2 -->|"may invoke"| cmd31
    who2 -->|"may invoke"| cmd33
    who2 -->|"may invoke"| cmd34
    who3 -->|"may invoke"| cmd32
    who4 -->|"may invoke"| cmd37
    who4 -->|"may invoke"| cmd38
    who5 -->|"may invoke"| cmd35
    who5 -->|"may invoke"| cmd36
    who5 -->|"may invoke"| cmd39
    who6 -->|"may invoke"| cmd40
    who6 -->|"may invoke"| cmd41
    who6 -->|"may invoke"| cmd42
    who6 -->|"may invoke"| cmd43
    who6 -->|"may invoke"| cmd44
    who6 -->|"may invoke"| cmd45
    who6 -->|"may invoke"| cmd46
    who6 -->|"may invoke"| cmd47
    cmd0 -->|"completed"| evt0
    cmd1 -->|"completed"| evt0
    cmd2 -->|"completed"| evt0
    cmd3 -->|"completed"| evt0
    cmd4 -->|"completed"| evt0
    cmd5 -->|"completed"| evt0
    cmd6 -->|"completed"| evt0
    cmd7 -->|"completed"| evt0
    cmd8 -->|"completed"| evt0
    cmd9 -->|"completed"| evt0
    cmd10 -->|"completed"| evt0
    cmd11 -->|"completed"| evt0
    cmd12 -->|"completed"| evt0
    cmd13 -->|"completed"| evt0
    cmd14 -->|"completed"| evt0
    cmd15 -->|"completed"| evt0
    cmd16 -->|"completed"| evt0
    cmd17 -->|"completed"| evt0
    cmd18 -->|"completed"| evt0
    cmd19 -->|"completed"| evt0
    cmd20 -->|"completed"| evt0
    cmd21 -->|"completed"| evt0
    cmd22 -->|"completed"| evt0
    cmd23 -->|"completed"| evt0
    cmd24 -->|"completed"| evt0
    cmd25 -->|"completed"| evt0
    cmd26 -->|"completed"| evt0
    cmd27 -->|"completed"| evt0
    cmd28 -->|"approved"| evt2
    cmd29 -->|"created"| evt7
    cmd30 -->|"denied"| evt3
    cmd31 -->|"pinned"| evt6
    cmd32 -->|"recorded"| evt4
    cmd33 -->|"removed"| evt5
    cmd34 -->|"revoked"| evt8
    cmd35 -->|"closed"| evt9
    cmd36 -->|"started"| evt11
    cmd37 -->|"observed"| evt10
    cmd38 -->|"observed"| evt10
    cmd39 -->|"started"| evt11
    cmd40 -->|"completed"| evt19
    cmd41 -->|"completed"| evt19
    cmd42 -->|"completed"| evt19
    cmd43 -->|"completed"| evt19
    cmd44 -->|"completed"| evt19
    cmd45 -->|"completed"| evt19
    cmd46 -->|"completed"| evt19
    cmd47 -->|"completed"| evt19
```

A command is accepted by the component that owns its context, emits the events one of its outcomes declares, and a dashed edge is a binding carrying an event into the next command. Design §9 begins one step earlier, at the actor who invokes the first command, and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with no edge at all may invoke nothing — which is something the model says, not an arrow somebody forgot.

## Bounded contexts

- **[Coding session](domains/agentide-coding.md)** (`agentide.coding`) — Semantic observation, change, execution, collaboration, evidence, and delivery intents. 11 types, no entities, no views, 28 commands, two events, one error and one actor.
- **[Session coordination](domains/agentide-coordination.md)** (`agentide.coordination`) — Durable hosted collaboration references stored by Service SDK without project file bytes. 10 types, three entities, three views, seven commands, seven events, two errors and three actors.
- **[Sessions](domains/agentide-session.md)** (`agentide.session`) — The durable identity and observable projection of one coding session. 10 types, one entity, one view, five commands, three events, two errors and two actors.
- **[Workspace surface](domains/agentide-surface.md)** (`agentide.surface`) — A renderer-neutral virtual workbench shared by agents, the browser, CLI snapshots, and the console TUI. Four types, one entity, one view, eight commands, eight events, one error and one actor.

## Components

A component is a unit of ownership, not a deployment. How many of each runs, and what each needs, is [the topology](topology.md).

**`agentide-engine`** — Validates intents, plans effects, obtains authority, journals, dispatches, and projects sessions. It owns [`agentide.coding`](domains/agentide-coding.md), [`agentide.coordination`](domains/agentide-coordination.md), [`agentide.session`](domains/agentide-session.md) and [`agentide.surface`](domains/agentide-surface.md). It accepts `agentide.coding.ApplyDeployment`, `agentide.coding.CancelProcess`, `agentide.coding.CreateCode`, `agentide.coding.CreateWorktree`, `agentide.coding.CutRelease`, `agentide.coding.DelegateAgent`, `agentide.coding.DeleteCode`, `agentide.coding.EditCode`, `agentide.coding.FinishWorktree`, `agentide.coding.InputProcess`, `agentide.coding.ListTerminals`, `agentide.coding.MessageAgent`, `agentide.coding.ObserveAgents`, `agentide.coding.ObserveChanges`, `agentide.coding.ObserveDeployment`, `agentide.coding.ObserveProcess`, `agentide.coding.ObserveWorktree`, `agentide.coding.OpenInteractiveTerminal`, `agentide.coding.PublishCode`, `agentide.coding.ReadCode`, `agentide.coding.RecordEvidence`, `agentide.coding.RenameCode`, `agentide.coding.SearchCode`, `agentide.coding.StartProcess`, `agentide.coding.TerminateTerminal`, `agentide.coding.VerifyCode`, `agentide.coding.WaitAgent`, `agentide.coding.WaitProcess`, `agentide.coordination.ApproveCheckpoint`, `agentide.coordination.CreateGrant`, `agentide.coordination.DenyCheckpoint`, `agentide.coordination.PinContext`, `agentide.coordination.RecordApprovalCheckpoint`, `agentide.coordination.RemoveContextPin`, `agentide.coordination.RevokeGrant`, `agentide.session.CloseSession`, `agentide.session.EnsureHostedSession`, `agentide.session.ReadEvents`, `agentide.session.SnapshotSession`, `agentide.session.StartSession`, `agentide.surface.CloseFile`, `agentide.surface.ClosePane`, `agentide.surface.FocusPane`, `agentide.surface.MoveCursor`, `agentide.surface.OpenFile`, `agentide.surface.OpenPane`, `agentide.surface.ShowDiff` and `agentide.surface.SnapshotSurface`. It publishes `agentide.coding.IntentCompleted`, `agentide.coding.IntentRefused`, `agentide.coordination.ApprovalCheckpointApproved`, `agentide.coordination.ApprovalCheckpointDenied`, `agentide.coordination.ApprovalCheckpointRecorded`, `agentide.coordination.ContextPinRemoved`, `agentide.coordination.ContextPinned`, `agentide.coordination.GrantCreated`, `agentide.coordination.GrantRevoked`, `agentide.session.SessionClosed`, `agentide.session.SessionObserved`, `agentide.session.SessionStarted`, `agentide.surface.CursorMoved`, `agentide.surface.DiffShown`, `agentide.surface.FileClosed`, `agentide.surface.FileOpened`, `agentide.surface.PaneClosed`, `agentide.surface.PaneFocused`, `agentide.surface.PaneOpened` and `agentide.surface.SurfaceObserved`.

## The other pages

| page | what is on it |
|---|---|
| [Coding session](domains/agentide-coding.md) | the `agentide.coding` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Session coordination](domains/agentide-coordination.md) | the `agentide.coordination` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Sessions](domains/agentide-session.md) | the `agentide.session` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Workspace surface](domains/agentide-surface.md) | the `agentide.surface` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Interactions](interactions.md) | every binding, with what it guarantees and what happens when it fails |
| [Type crossings](crossings.md) | every conversion this system permits, and the reason someone gave for it |
| [Topology](topology.md) | what each component needs in order to run |


---

Generated from agentide v1 · model digest `360a1e3f4110754181c30d72530c91ae4344a5f5d4c3aff968bda22d67ca12f3` · contract digest `90e1687e023e28f77eca4934db634944dcaf13a22a2075c642452ec5a97822b4`. Do not edit this file; change the specification and regenerate it with `ess generate`.
