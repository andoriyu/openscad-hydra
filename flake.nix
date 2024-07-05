{
  description = "Minimal Rust Development Environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    andoriyu = {
      url = "github:andoriyu/flakes";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        fenix.follows = "fenix";
      };
    };
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    andoriyu,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      cwd = builtins.toString ./.;
      overlays = [ fenix.overlays.default ];
      pkgs = import nixpkgs {inherit system overlays;};
      control-plane = pkgs.callPackage ./nix/control-plane {inherit nixpkgs system fenix andoriyu;};
    in
      with pkgs; {
        formatter = nixpkgs.legacyPackages.${system}.alejandra;
        devShell = gccMultiStdenv.mkDerivation rec {
          name = "rust";
          nativeBuildInputs = [
            (with fenix.packages.${system};
              combine [
                (latest.withComponents [
                  "cargo"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
              ])
            bacon
            binutils
            cargo-generate
            cargo-cache
            cargo-deny
            cargo-diet
            cargo-nextest
            cargo-outdated
            cargo-sort
            cargo-sweep
            cargo-wipe
            cargo-workspaces
            cmake
            curl
            gnumake
            jq
            just
            pkg-config
            rust-analyzer-nightly
            zlib
            openscad-unstable
            asciinema
          ];
        };
      });
}
