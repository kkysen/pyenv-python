use pyenv_python::pyenv::action::Action;

/// Print the current `python` path as determined by `pyenv`.
fn main() {
    Action::from_args().run()
}
