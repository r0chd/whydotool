{
  rustPlatform,
  lib,
  pkg-config,
  libxkbcommon,
  wayland,
  pipewire,
  portals ? true,
  llvmPackages_20,
  clangStdenv,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage.override { stdenv = clangStdenv; } {
  pname = "whydotool";
  inherit (cargoToml.package) version;

  cargoLock.lockFile = ../Cargo.lock;

  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      let
        relPath = lib.removePrefix (toString ../. + "/") (toString path);
      in
      lib.any (p: lib.hasPrefix p relPath) [
        "src"
        "Cargo.toml"
        "Cargo.lock"
      ];
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    libxkbcommon
    wayland
  ]
  ++ lib.optionals portals [
    pipewire
  ];

  LIBCLANG_PATH = "${llvmPackages_20.libclang.lib}/lib";

  buildNoDefaultFeatures = true;
  cargoFeatures = lib.optionals portals [ "portals" ];

  meta = {
    description = "Wayland-native command-line automation tool.";
    homepage = "https://forgejo.r0chd.pl/r0chd/whydotool";
    license = lib.licenses.mit;
    maintainers = builtins.attrValues { inherit (lib.maintainers) r0chd; };
    platforms = lib.platforms.linux;
    mainProgram = "whydotool";
  };
}
