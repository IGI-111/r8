{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
        libPath = with pkgs; lib.makeLibraryPath [ SDL2 ];

      in rec {
        # `nix build`
        packages.r8 = naersk-lib.buildPackage {
          pname = "r8";
          root = ./.;
          buildInputs = with pkgs; [ SDL2 ];
          nativeBuildInputs = with pkgs; [ makeWrapper ];
          postInstall = ''
            wrapProgram "$out/bin/r8" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';

        };
        defaultPackage = packages.r8;

        # `nix run`
        apps.r8 = flake-utils.lib.mkApp { drv = packages.r8; };
        defaultApp = apps.r8;

        # `nix develop`
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            rustPackages.clippy
            rust-analyzer

            SDL2
          ];
          LD_LIBRARY_PATH = libPath;
        };
      });
}
