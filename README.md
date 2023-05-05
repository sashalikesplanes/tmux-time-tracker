A utility based on tmux hooks in order to track time attached to each session

## Install:
1. Clone the repository
2. Run `cargo build --release`
3 .Move `./target/release/tmux-time-tracker` into you $PATH
4. Add the following to your tmux.conf:
    ```bash
    set-hook -g client-attached 'run-shell "output=$(tmux-time-tracker geth #{session_name}); tmux-time-tracker attach #{session_name}; tmux display-message \"Total time: \$output\""'
    set-hook -g client-detached 'run-shell "tmux-time-tracker detach"'
    set-hook client-session-changed 'run-shell "tmux-time-tracker detach; tmux-time-tracker attach #{session_name}; output=$(tmux-time-tracker geth #{session_name}); tmux display-message \"Total time: \$output\""'
    ```

WARNING: does not work correctly if you attach to the same tmux instance from multiple clients
