use histrion::Workspace;

#[test]
fn example_script() {
    use histrion::action::*;
    use histrion::time::*;

    let flag = Flag {
        head: "arrived".into(),
        body: vec![].into(),
    };

    let script = Action::Block {
        body: vec![
            WaitExpr::Flag {
                head: "arrived".into(),
                args: vec![].into(),
            }.and_then(Action::Halt),
            Action::CreateActor { name: "Mars".into(), },
            Action::SetTrajectory {
                name: "Mars".into(),
                value: TrajectoryExpr::Linear {
                    velocity: (1e-3, 0.0, 0.0).into(),
                }.into(),
            },
            WaitExpr::Delay { interval: Interval::from_f64(5.0), }
                .and_then(Action::Block {
                    body: vec![
                        Action::Fulfill { flag },
                    ].into(),
                }),
        ].into(),
    };

    let mut workspace = Workspace::new();
    workspace.perform(&script).unwrap();
    workspace.simulate().unwrap();
}
