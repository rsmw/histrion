use histrion::Workspace;

#[test]
fn example_script() {
    use histrion::action::*;
    use histrion::script::*;

    let script = vec![
        Action::Spawn {
            name: "Mars".into(),
        },
        Action::AsActor {
            name: "Mars".into(),
            script: vec![
                Action::Trace {
                    comment: "Waiting 1 hour...".into(),
                },
                Action::Wait {
                    interval: TimeExpr::Constant {
                        number: 1.0,
                        unit: TimeUnit::Hour,
                    }.into(),
                },
                Action::Trace {
                    comment: "Sending #arrived()".into(),
                },
                Action::Transmit {
                    head: "arrived".into(),
                    args: vec![].into(),
                },
            ].into()
        },

        Action::ListenFor {
            head: "arrived".into(),
            args: vec![].into(),
        },
        Action::Trace {
            comment: "OK, time to halt".into(),
        },
        Action::Halt,
    ].into();

    let mut workspace = Workspace::new();
    workspace.perform(script).unwrap();
    workspace.simulate().unwrap();
}
