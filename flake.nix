{
  description = "Heimdall dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      rec {
        devShells.default = pkgs.mkShellNoCC {
          packages = with pkgs; [
            cmake
            flatbuffers
            git
            libiconv
            lld
            llvm
            rustup
            zsh
          ];

          shell = "${pkgs.zsh}/bin/zsh";

          shellHook = ''
            export LIBRARY_PATH="${pkgs.libiconv}/lib:$LIBRARY_PATH"
            export C_INCLUDE_PATH="${pkgs.libiconv}/include:$C_INCLUDE_PATH"
            export CPLUS_INCLUDE_PATH="${pkgs.libiconv}/include:$CPLUS_INCLUDE_PATH"
            if [ -f "$HOME/.zshrc" ]; then
              exec "${pkgs.zsh}/bin/zsh" -l
            fi
          '';
        };
      }
    );
}
