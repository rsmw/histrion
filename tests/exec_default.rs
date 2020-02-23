use histrion::Workspace;

#[test]
fn example_script() {
    use histrion::script::*;

    let script = Script::new(vec![
        Stmt::CreateActor { name: "Mars".into(), },
        Stmt::AsActor {
            name: "Mars".into(),
            body: vec![
                Stmt::Wait {
                    interval: TimeExpr::Constant {
                        number: 1.0,
                        unit: TimeUnit::Hour,
                    },
                },
                Stmt::Transmit {
                    name: "arrived".into(),
                    args: vec![].into(),
                },
            ].into()
        },

        Stmt::ListenFor {
            name: "arrived".into(),
            args: vec![].into(),
        },
        Stmt::Trace {
            comment: "OK, time to halt".into(),
        },
        Stmt::Halt,
    ].into()).to_action();

    let mut workspace = Workspace::new();
    workspace.perform(&script).unwrap();
    workspace.simulate().unwrap();
}
