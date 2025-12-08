{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              just
              bash
              parallel
              rustup
              gcc
              pnpm
              nodejs
              wasm-pack
              zip
              wasm-bindgen-cli_0_2_100
            ];

            shellHook = ''
              echo "
                                                                   YSOKGECAACDFIMRVZ               
                                                                YQHAAAAABDFFDBAAAAABU              
                                                              WKAAAFNTWZ      ZWTQNNY              
                         ZTPMIFDCAAAAAEJPUZ                 ZKAADOX                                
                      ZTJBAAABFHJMMMMLFAAAAGRZ             YFAAKZ                                  
                     XDAAELSW          ZVQIAADQZ           QAAL                                    
                     ZPNW                  VIAACOZ          VTZ                                    
                          YPLIGDBBDFIKPY     VIABX                   YOKIFDABDGILPY                
                        TJAAABEILLIFCAAAJT     WW                 ZSIAAACFIMLHEBAABKU              
                      UEAADNW        XNEAADT                     SDAAEOX        WMDAAEV            
                 YTTTNAACT              UDAAM                  ZLAAEV              TCAAOTTTY       
                WBAAAAAFX                YGAAM                 KAAHY                XEAAAAACX      
                YAAAAACZ                  ZEAAJMMMMMMIGIMMMMMMIAAFZ                  YBAAAAAZ      
                YAAAAAK                    JAAAAAAAAAAAAAAAAAAAAAL                    IAAAAAZ      
                 TNDAAT                    TAAAAAAAAAAAAAAAAAAAAAV                    RAAENU       
                   KAAM                    NAAGTTTTTTRORTTTTTTFAAP                    KAAM         
                   UAAC                    EAAU               RAAF                   ZBAAW         
                    NAAHX                XIAAM                 KAAJY                WGAAO          
                     OAABNX            XOBAAM                  ZLAACOY            XMAAAP           
                      UEAAAGQVY    ZVQHAAAET                     SDAAAHRVZ    YVPGAAAFV            
                        VNDAAAABEFBAAAADMV                         UMCAAAABFEBAAAAENW              
                           VRPMKHHJMORV                               UROMJHHKMPRW
              "
            '';
          };
      }
    );
}
