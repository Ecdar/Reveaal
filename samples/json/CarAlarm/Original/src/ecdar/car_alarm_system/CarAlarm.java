package ecdar.car_alarm_system;

import java.time.Instant;

class CarAlarm {
    //Inputs
    private static final String INPUT_CLOSE = "close";
    private static final String INPUT_OPEN = "open";
    private static final String INPUT_LOCK = "lock";
    private static final String INPUT_UNLOCK = "unlock";

    //Outputs
    private static final String OUTPUT_ARMED_OFF = "armedOff";
    private static final String OUTPUT_ARMED_ON = "armedOn";
    private static final String OUTPUT_FLASH_OFF = "flashOff";
    private static final String OUTPUT_FLASH_ON = "flashOn";
    private static final String OUTPUT_SOUND_OFF = "soundOff";
    private static final String OUTPUT_SOUND_ON = "soundOn";

    public enum location {L0, L1, L2, L3, L6, L9, L11, L12, L13, L5, L10, L4, Done}

    Instant clockX;
    private boolean alarmLocked;
    location nextLocation;


    CarAlarm(){
    }

    void update() {

        switch (nextLocation) {
            case L0:
                nextLocation = L0();
                break;
            case L1:
                nextLocation = L1();
                break;
            case L2:
                nextLocation = L2();
                break;
            case L3:
                nextLocation = L3();
                break;
            case L4:
                nextLocation = L4();
                break;
            case L5:
                nextLocation = L5();
                break;
            case L6:
                nextLocation = L6();
                break;
            case L9:
                nextLocation = L9();
                break;
            case L10:
                nextLocation = L10();
                break;
            case L11:
                nextLocation = L11();
                break;
            case L12:
                nextLocation = L12();
                break;
            case L13:
                nextLocation = L13();
                break;
            case Done:
                System.exit(1);
                break;
            default:
                new RuntimeException("Location " + nextLocation.toString() + " not expected").printStackTrace();
                System.exit(1);
        }
    }



    private location L0() {
        if (CarAlarmHelper.inputReady()) {
            String input = CarAlarmHelper.read();

            if(input.equals(INPUT_CLOSE)) {
                return location.L1;
            } else if(input.equals(INPUT_LOCK)) {
                return location.L2;
            }
        } else {
            CarAlarmHelper.stepDone = true;
            return location.L0;
        }

        return location.Done;
    }

    private location L1() {
        if (CarAlarmHelper.inputReady()) {
            String input = CarAlarmHelper.read();

            if(input.equals(INPUT_OPEN)) {
                return location.L0;
            } else if(input.equals(INPUT_LOCK)) {
                clockX = CarAlarmHelper.handler.resetTime();
                return location.L3;
            }
        } else {
            CarAlarmHelper.stepDone = true;
            return location.L1;
        }

        return location.Done;
    }

    private location L2() {
        if (CarAlarmHelper.inputReady()) {
            String input = CarAlarmHelper.read();

            if(input.equals(INPUT_UNLOCK)) {
                return location.L0;
            } else if(input.equals(INPUT_CLOSE)) {
                clockX = CarAlarmHelper.handler.resetTime();
                return location.L3;
            }
        } else {
            CarAlarmHelper.stepDone = true;
            return location.L2;
        }

        return location.Done;
    }

    private location L3() {
        boolean timeOk = CarAlarmHelper.getValue(clockX) < 20.0;
        if (timeOk) {
            if (CarAlarmHelper.inputReady()) {
                final String input = CarAlarmHelper.read();

                if(input.equals(INPUT_UNLOCK)) {
                    return location.L1;
                } else if(input.equals(INPUT_OPEN)) {
                    return location.L2;
                }
            } else {
                CarAlarmHelper.stepDone = true;
                return location.L3;
            }
        } else {
            CarAlarmHelper.write(OUTPUT_ARMED_ON);
            alarmLocked = false;
            return location.L4;
        }
        return location.Done;
    }

    private location L4() {
        if (CarAlarmHelper.inputReady()) {
            String input = CarAlarmHelper.read();

            if(input.equals(INPUT_UNLOCK)) {
                clockX = CarAlarmHelper.handler.resetTime();
                return location.L5;
            } else if(input.equals(INPUT_OPEN)) {
                clockX = CarAlarmHelper.handler.resetTime();
                return location.L6;
            }
        } else if (alarmLocked && CarAlarmHelper.getValue(clockX) > 400) {
            clockX = CarAlarmHelper.handler.resetTime();
            CarAlarmHelper.write(OUTPUT_ARMED_OFF);
            return location.L1;
        } else {
            CarAlarmHelper.stepDone = true;
            return location.L4;
        }

        return location.Done;
    }

    private location L5(){
        CarAlarmHelper.write(OUTPUT_ARMED_OFF);
        return location.L1;
    }

    private location L6(){
        CarAlarmHelper.write(OUTPUT_ARMED_OFF, OUTPUT_FLASH_ON, OUTPUT_SOUND_ON);
        return location.L9;
    }

    private location L9() {
        if(CarAlarmHelper.getValue(clockX) <= 30) {
            if(CarAlarmHelper.inputReady()) {
                if (CarAlarmHelper.read().equals(INPUT_UNLOCK)) {
                    clockX = Instant.now();
                    return location.L10;
                }
            } else {
                CarAlarmHelper.stepDone = true;
                return location.L9;
            }
        } else {
            CarAlarmHelper.write(OUTPUT_SOUND_OFF);
            return location.L11;
        }
        //Error happend
        return location.Done;
    }

    private location L10(){
        CarAlarmHelper.write(OUTPUT_SOUND_OFF);
        CarAlarmHelper.write(OUTPUT_FLASH_OFF);
        return location.L0;
    }

    private location L11(){
        if (CarAlarmHelper.handler.getValue(clockX) <= 300.0){
            if(CarAlarmHelper.inputReady()) {
                if (CarAlarmHelper.read().equals(INPUT_UNLOCK)) {
                    clockX = Instant.now();
                    return location.L10;
                }
            } else {
                CarAlarmHelper.stepDone = true;
                return location.L11;
            }
        } else {
            CarAlarmHelper.write(OUTPUT_SOUND_OFF);
            CarAlarmHelper.write(OUTPUT_FLASH_OFF);
            return location.L12;
        }

        // Error happened
        return location.Done;
    }

    private location L12() {
        if (CarAlarmHelper.inputReady()) {
            String input = CarAlarmHelper.read();
            if (input.equals(INPUT_CLOSE)) {
                clockX = CarAlarmHelper.handler.resetTime();
                return location.L13;
            } else if (input.equals(INPUT_UNLOCK)) {
                return location.L0;
            }
        }
        CarAlarmHelper.stepDone = true;
        return location.L12;
    }

    private location L13(){
        CarAlarmHelper.write(OUTPUT_ARMED_ON);
        alarmLocked = true;
        return location.L4;
    }
}