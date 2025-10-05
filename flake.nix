{
  inputs.tooling.url = "github:mox-desktop/tooling";

  outputs =
    { self, tooling, ... }:
    tooling.lib.mkMoxFlake {
      devShells = tooling.lib.forAllSystems (pkgs: {
        default = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } (
          pkgs.lib.fix (finalAttrs: {
            buildInputs = builtins.attrValues {
              inherit (pkgs)
                rustToolchain
                rust-analyzer-unwrapped
                nixd
                libxkbcommon
                pkg-config
                pipewire
                libclang
                ;
            };
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath finalAttrs.buildInputs;
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          })
        );
      });

      packages = tooling.lib.forAllSystems (pkgs: {
        whydotool = pkgs.callPackage ./nix/package.nix {
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rustToolchain;
            rustc = pkgs.rustToolchain;
          };
        };
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.whydotool;
      });

      nixosModules = {
        whydotool = import ./nix/nixos.nix;
        default = self.nixosModules.whydotool;
      };
    };
}
