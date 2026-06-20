{
  description = "Claude and Codex rate limit status CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    in
    {
      packages = builtins.listToAttrs (
        map (
          system:
          let
            pkgs = import nixpkgs { inherit system; };
            wabi = pkgs.rustPlatform.buildRustPackage {
              pname = "wabi";
              version = "0.1.0";
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };
            };
          in
          {
            name = system;
            value = {
              inherit wabi;
              default = wabi;
            };
          }
        ) systems
      );
    };
}
