{
  rustPlatform,
  lib,
  pkg-config,
  libxkbcommon,
  wayland,
  pipewire,
  ydotoolCompat ? false,
  portals ? false,
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
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

  cargoFeatures = lib.optionals portals [ "portals" ];

  postInstall = lib.optionalString ydotoolCompat ''
    ln -s $out/bin/whydotool $out/bin/ydotool
  '';

  meta = {
    description = "Wayland-native command-line automation tool.";
    homepage = "https://github.com/r0chd/whydotool";
    license = lib.licenses.mit;
    maintainers = builtins.attrValues { inherit (lib.maintainers) r0chd; };
    platforms = lib.platforms.linux;
    mainProgram = "whydotool";
  };
}
