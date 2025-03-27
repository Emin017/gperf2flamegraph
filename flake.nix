{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      treefmt-nix,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        treefmtEval = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "Cargo.toml";
          programs = {
            nixfmt.enable = true; # nix
            rustfmt.enable = true; # rust
            yamlfmt.enable = true; # yaml
            taplo.enable = true; # toml
          };
        };
      in
      {
        packages = {
          gperf2flamegraph = pkgs.callPackage ./nix/pkgs/gperf2flamegraph.nix { };
          default = self.packages.${system}.gperf2flamegraph;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.gperf2flamegraph ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
        formatter = treefmtEval.config.build.wrapper;
        checks = {
          formatting = treefmtEval.config.build.check self;
        };
      }
    );
}
