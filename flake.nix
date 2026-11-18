{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" ];
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      forAll = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in {
      packages = forAll (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.sqlite ];
          nativeCheckInputs = [ pkgs.git pkgs.jujutsu ];
          # tests need a writable home and a vcs identity
          preCheck = ''
            export HOME=$TMPDIR
            export JJ_USER=test JJ_EMAIL=test@example.com
            export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.com
            export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.com
          '';
          meta.mainProgram = "party";
        };
      });
    };
}

