{
  description = "Nix + WebAssembly example project";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1.*.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "https://flakehub.com/f/nix-community/naersk/0.1.*.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, ... }@inputs:
    let
      pkgName = (self.lib.fromToml ./Cargo.toml).package.name;
      supportedSystems = [ "aarch64-darwin" "aarch64-linux" "x86_64-darwin" "x86_64-linux" ];
      forAllSystems = f: inputs.nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import inputs.nixpkgs { inherit system; overlays = [ self.overlays.default ]; };
        inherit system;
      });
      rustWasmTarget = "wasm32-unknown-unknown";
    in
    {
      overlays.default = final: prev: rec {
        system = final.stdenv.hostPlatform.system;

        # Builds a Rust toolchain from rust-toolchain.toml
        rustToolchain = with inputs.fenix.packages.${system};
          combine [
            latest.rustc
            latest.cargo
            targets.${rustWasmTarget}.latest.rust-std
          ];

        buildRustWasm = self.lib.buildRustWasm final;
        buildWasmPackage = self.lib.buildWasmPackage final;
        buildRustServerExec = self.lib.buildRustServerExec final;
      };

      # Development environments
      devShells = forAllSystems ({ pkgs, system }: {
        default =
          let
            helpers = with pkgs; [ direnv jq ];
          in
          pkgs.mkShell {
            packages = helpers ++ (with pkgs; [
              rustToolchain # cargo, etc.
              wabt # WebAssembly Binary Toolkit
              wasmedge # Wasm runtime
              wasmtime # Wasm runtime
              cargo-edit # cargo add, cargo rm, etc.
              tree # for visualizing results
              alsa-lib.dev
            ]);
          };
      });

      packages = forAllSystems ({ pkgs, system }: rec {
        default = hello-wasm;
        hello-wasm = pkgs.buildRustWasm {
          name = "mothers-day-mini-game";
          src = self;
        };
        hello-wasm-pkg = pkgs.buildWasmPackage { };
        hello-wasm-server-exec = pkgs.buildRustServerExec { };
      });

      lib = {
        # Helper function for reading TOML files
        fromToml = file: builtins.fromTOML (builtins.readFile file);

        handleArgs =
          { name ? null
          , src ? self
          , cargoToml ? ./Cargo.toml
          }:
          let
            meta = (self.lib.fromToml ./Cargo.toml).package;
            pkgName = if name == null then meta.name else name;
            pkgSrc = builtins.path { path = src; name = "${pkgName}-source"; };
          in
          {
            inherit (meta) name;
            inherit pkgName;
            src = pkgSrc;
            inherit cargoToml;
          };

        buildRustWasm = pkgs: { name, src }:
          let
            naerskLib = pkgs.callPackage inputs.naersk {
              cargo = pkgs.rustToolchain;
              rustc = pkgs.rustToolchain;
            };
          in
          naerskLib.buildPackage {
            inherit name src;
            CARGO_BUILD_TARGET = rustWasmTarget;
            buildInputs = with pkgs; [ wabt alsa-lib.dev ];
          };

        # only works with wasm32-unknown-unknown ?
        buildRustServerExec = pkgs: args:
          let
            finalArgs = self.lib.handleArgs args;
            wasmPkg = self.lib.buildRustWasm pkgs {
              inherit (finalArgs) name src;
            };
          in
          pkgs.stdenv.mkDerivation rec {
            name = finalArgs.name;
            src = finalArgs.src;
            nativeBuildInputs = with pkgs; [ makeWrapper ];
            installPhase = ''
              makeWrapper wasm-server-runner ${wasmPkg}/${finalArgs.name}.wasm
            '';
          };

        # Take a Wasm binary and strip it, provide stats, etc.
        buildWasmPackage = pkgs: args:
          let
            finalArgs = self.lib.handleArgs args;
            wasmPkg = self.lib.buildRustWasm pkgs {
              inherit (finalArgs) name src;
            };
          in
          pkgs.stdenv.mkDerivation {
            name = finalArgs.name;
            src = finalArgs.src;
            buildInputs = with pkgs; [
              # includes wasm-strip, wasm2wat, wasm-stats, wasm-objdump, and wasm-validate
              wabt
            ];
            buildPhase = ''
              mkdir -p $out/{lib,share}
              cp ${wasmPkg}/lib/${finalArgs.name}.wasm $out/lib/${finalArgs.pkgName}.wasm
              wasm2wat $out/lib/${finalArgs.pkgName}.wasm > $out/share/${finalArgs.pkgName}.wat
              wasm-stats $out/lib/${finalArgs.pkgName}.wasm -o $out/share/${finalArgs.pkgName}.dist
              wasm-objdump \
                --details $out/lib/${finalArgs.pkgName}.wasm > $out/share/${finalArgs.pkgName}-dump.txt
            '';
            doCheck = true;
          };
      };
    };
}
