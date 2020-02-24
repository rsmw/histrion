use histrion::Workspace;

#[test]
fn example_script() {
    use histrion::script::*;

    let script = Script::default().into_inner();

    let mut workspace = Workspace::new();
    workspace.perform(script).unwrap();
    workspace.simulate().unwrap();
}
