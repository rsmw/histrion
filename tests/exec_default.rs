use histrion::Workspace;

#[test]
fn exec_default() {
    let mut workspace = Workspace::new();

    workspace.simulate();
}

#[test]
fn example_script() {
    use histrion::action::*;
    use histrion::time::*;

    let script = Action::Block {
        body: vec![
            Action::CreateActor { name: "Mars".into(), },
            Action::SetTrajectory {
                name: "Mars".into(),
                value: TrajectoryExpr::Linear {
                    velocity: (1e-3, 0.0, 0.0).into(),
                }.into(),
            },
            WaitExpr::Delay { interval: Interval::from_f64(5.0), }
                .and_then(Action::Halt),
        ].into(),
    };

    let mut workspace = Workspace::new();
    workspace.perform(&script);
    workspace.simulate();
}
