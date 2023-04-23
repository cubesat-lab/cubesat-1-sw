/*
C4 Model:
    https://c4model.com/
    Context, Containers, Components, and Code

Structurizr:
    https://structurizr.com/

Structurizr DSL:
    https://github.com/structurizr/dsl/blob/master/docs/language-reference.md

Open this model in browser:
    1. Start Docker
    2. Run Task -> structurizr
    3. http://localhost:8080/
*/

workspace {

    model {
        user = person "Satellite Operator"
        space = element "Space"
        earth = element "Earth"
        sun = element "Sun"

        cubesat_system = softwareSystem "CubeSat System" {
            gs_app = container "GS Application" "Ground Station" "Python"
            gs_fw = container "GS Firmware" "Ground Station" "Rust"
            adcs_fw = container "ADCS Firmware" "Attitude Determination and Control System" "Rust"
            eps_fw = container "EPS Firmware" "Electronic Power System" "Rust"
            obc_fw = container "OBC Firmware" "On-Board Computer" "Rust" {
                can = component "CAN"
                tmtc = component "TMTC"
                mode_mgr = component "Mode Manager"
                mem = component "MEM"
                rf = component "RF"
                pus = component "PUS"
            }
            payload = container "Payload" "Generic Payload System"
        }

        # relationships between people, environment and software system
        user -> cubesat_system "Operates"
        cubesat_system -> space "Operates in"
        cubesat_system -> earth "Orbits"
        sun -> cubesat_system "Illuminate"

        # relationships to/from containers
        user -> gs_app "Sends TCs, receives TMs"
        gs_app -> gs_fw "Exchange TCs and TMs over USB"
        adcs_fw -> sun "Detects attitude relative to"
        adcs_fw -> earth "Detects attitude relative to"
        adcs_fw -> space "Detects & controls attitude in"
        eps_fw -> sun "Receives power from"
        gs_fw -> obc_fw "Exchange TCs and TMs over UHF"
        obc_fw -> adcs_fw "Controls"
        obc_fw -> eps_fw "Controls"
        obc_fw -> payload "Controls"

        # relationships to/from components
        gs_fw -> rf "Control & telemetry over UHF"
        can -> adcs_fw "Control & telemetry over CAN"
        can -> eps_fw "Control & telemetry over CAN"
        can -> payload "Control & telemetry over CAN"
    }

    views {
        systemContext cubesat_system {
            include *
            autolayout lr
            description "CubeSat System Context"
        }

        container cubesat_system  {
            include *
            autolayout lr
            description "CubeSat System Containers"
        }

        component obc_fw {
            include *
            autolayout lr
            description "On-Board Computer Container"
        }

        theme default
    }

}