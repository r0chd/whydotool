{
  inputs.tooling.url = "github:mox-desktop/tooling";

  outputs =
    { self, tooling, ... }:
    tooling.lib.mkMoxFlake {
      devShells = tooling.lib.forAllSystems (pkgs: {
        default = pkgs.mkShell (
          pkgs.lib.fix (finalAttrs: {
            buildInputs = builtins.attrValues {
              inherit (pkgs)
                rustToolchain
                rust-analyzer-unwrapped
                nixd
                wayland
                just
                ;
            };
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath finalAttrs.buildInputs;
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          })
        );
      });

      packages = tooling.lib.forAllSystems (pkgs: {
        clicker-rs = pkgs.callPackage ./nix/package.nix {
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rustToolchain;
            rustc = pkgs.rustToolchain;
          };
        };
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.clicker-rs;
      });

      nixosModules = {
        clicker-rs = import ./nix/nixos.nix;
        default = self.nixosModules.clicker-rs;
      };
    };
}
