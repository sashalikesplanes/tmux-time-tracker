A utility based on tmux hooks in order to track time attached to each session

## Install:
1. Clone the repository
2. Run `cargo build --release`
3 .Move `./target/release/tmux-time-tracker` into you $PATH
4. Add the following to your tmux.conf:
    ```bash
    set-hook -g client-attached 'run-shell "tmux-time-tracker attached #{session_name}"'
    set-hook -g client-detached 'run-shell "tmux-time-tracker detached #{session_name}"'
    set-hook client-session-changed 'run-shell "tmux-time-tracker changed #{session_name}"'
    ```

WARNING: does not work correctly if you attach to the same tmux instance from multiple clients
