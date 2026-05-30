If you wish to contribute to this repository, please install the git hook scripts in `install-git-hooks`. Set up rustfmt so that it runs the nightly version `cargo +nightly fmt`, and uses the configuration file in `rustfmt.toml` (which should be automatic).

A few requirements:
* The style settings in `rustfmt.toml` aren't negotiable, but there may be some changes in the future as that tool improves and offers better configuration.
* The scripts above will enforce this, but before pushing, your repository is to be clean, clippy must be happy, and rustfmt must be complete.
* `.gitignore` is checked into the repository, this ensures noone accidentally checks in `target` or other build artifacts. 
  * If you have files of your own which you do not wish to be in the repository, you are welcome to create a local folder with your username, and add it to `.gitignore`.
  * `Cargo.lock` is not ignored, [despite this being a library](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control), because it is all but an executable. Also because it's hard to develop if you need to fix compatibility issues on a new checkout before you can start your fix. To address the problem in the link above, it's recommended that any libraries that *use* elbie also publish their use of elbie to the user (`pub use elbie`), so downstream users don't have to add elbie as a dependency to their own tool and can share your access. This is a practice I wish was more common in the rust community, it would make dependency trees so much cleaner.
