*** Settings ***
Library         OperatingSystem
Library         Process
Library         String
Resource        resources.robot
Suite Setup     Start QEMU And Capture Port
Suite Teardown  Terminate QEMU


*** Variables ***
${VIRTUAL_PORT}             ${EMPTY}
${QEMU_PID}                 ${EMPTY}
${OUTPUT_FILE}              ${CURDIR}/qemu_output.txt
${ERROR_FILE}               ${CURDIR}/qemu_error.txt


*** Test Cases ***
MostSimpleTest
    ${result}               Set Variable            Hello
    Log                     ${result}

SerialGoodMessage
    [Documentation]         Send a valid frame
    ${script_path} =        Set Variable            ${SCRIPTS_FOLDER}/good_frame.py
    ${result} =             Run Process             ${PYTHON}               ${script_path}          -p ${VIRTUAL_PORT}
    Log                     ${result.stdout}
    # Validate Length
    ${length} =             Get Regexp Matches      ${result.stdout}        Length:\\s*(\\d+)
    Should Not Be Empty     ${length}
    Should Be Equal         ${length}[0]            Length: 2
    # Validate Data
    ${useful_data} =        Get Regexp Matches      ${result.stdout}        Useful data:\\s+([a-f0-9]+)
    Should Not Be Empty     ${useful_data}
    Should Be Equal         ${useful_data}[0]       Useful data: cafe

SerialBadMessage
    [Documentation]         Send an invalid frame
    ${script_path} =        Set Variable            ${SCRIPTS_FOLDER}/bad_frame.py
    ${result} =             Run Process             ${PYTHON}               ${script_path}          -p ${VIRTUAL_PORT}
    Log                     ${result.stdout}
    # Validate Length
    ${length} =             Get Regexp Matches      ${result.stdout}        Length:\\s*(\\d+)
    Should Not Be Empty     ${length}
    Should Be Equal         ${length}[0]            Length: 2
    # Validate Data
    ${useful_data} =        Get Regexp Matches      ${result.stdout}        Useful data:\\s+([a-f0-9]+)
    Should Not Be Empty     ${useful_data}
    Should Be Equal         ${useful_data}[0]       Useful data: ffff


*** Keywords ***
Terminate QEMU
    Terminate Process       handle=${QEMU_PID}      kill=True
    Log                     QEMU process terminated

Start QEMU And Capture Port
    [Documentation]         Starts QEMU, captures its output to extract the virtual serial port, and sets it as a global variable.
    ${pid} =                Start Process           ${QEMU_COMMAND}         shell=True              stdout=${OUTPUT_FILE}   stderr=${ERROR_FILE}
    Set Global Variable     ${QEMU_PID}             ${pid}
    # Adjust the sleep time as needed to ensure QEMU has enough time to start and print the port information
    Sleep                   5s
    ${output} =             Get File                ${OUTPUT_FILE}
    ${errors} =             Get File                ${ERROR_FILE}
    ${combined_output} =    Catenate                SEPARATOR=${\n}         ${output}               ${errors}
    ${port} =               Fetch Port From Output  ${combined_output}
    Set Global Variable     ${VIRTUAL_PORT} ${port}
    Log                     QEMU started with PID: ${pid}. Virtual serial port: ${VIRTUAL_PORT}

Fetch Port From Output
    [Arguments]             ${output}
    Log                     My output is ${output}
    ${port_match} =         Get Regexp Matches      ${output}               \/dev\/pts/(\\d+)
    Log                     My port is ${port_match} =
    RETURN                  ${port_match}[0]
