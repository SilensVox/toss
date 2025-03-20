{
  description = "A throw-catch style move and copy program";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = nixpkgs.lib.genAttrs supportedSystems;
      
      # Function to get the Toss package for a system
      packageFor = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) self.overlays.default ];
          };
        in pkgs.toss;
      
      # Read package version from Cargo.toml
      version = "1.1.1"; # Use static version for simplicity
    in
    {
      overlays.default = final: prev: {
        toss = final.rustPlatform.buildRustPackage {
          pname = "toss";
          inherit version;
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with final; [
            pkg-config
          ];
          
          buildInputs = with final; [
            openssl
          ];
          
          meta = with final.lib; {
            description = "A throw-catch style move and copy program";
            homepage = "https://github.com/scientiac/toss";
            license = licenses.gpl3;
            maintainers = with maintainers; [ ];
          };
        };
      };
      
      # Packages for each supported system
      packages = forEachSupportedSystem (system: {
        default = packageFor system;
        toss = packageFor system;
      });
      
      # NixOS module
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.toss;
        in
        with lib; {
          options.programs.toss = {
            enable = mkEnableOption "toss program";
            package = mkOption {
              type = types.package;
              description = "The toss package to use.";
              default = self.packages.${pkgs.system}.toss;
            };
          };
          
          config = mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };
      
      # Home Manager module that doesn't depend on system-specific logic
      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.toss;
        in
        with lib; {
          options.programs.toss = {
            enable = mkEnableOption "toss program";
            package = mkOption {
              type = types.package;
              description = "The toss package to use.";
              default = self.packages.${pkgs.system}.toss;
            };
          };
          
          config = mkIf cfg.enable {
            home.packages = [ cfg.package ];
          };
        };
      
      # Development shell
      devShells = forEachSupportedSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          
          rustToolchain =
            let
              rust = pkgs.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default.override {
                extensions = [ "rust-src" "rustfmt" ];
              };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
            ];
            shellHook = ''
              echo "Developing Toss!"
            '';
            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            };
          };
        }
      );
    };
}
