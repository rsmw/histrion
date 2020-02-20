use histrion::Workspace;

fn main() {
    let mut workspace = Workspace::new();

    while !workspace.has_halted() {
        workspace.update();
    }
}
