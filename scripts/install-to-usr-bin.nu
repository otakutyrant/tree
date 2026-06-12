#!/usr/bin/env nu

let repo_dir = $env.FILE_PWD | path dirname

cd $repo_dir
do --capture-errors { ^cargo build --release }
do --capture-errors {
    ^sudo install -Dm755 ($repo_dir | path join target release tree) /usr/bin/tree
}
