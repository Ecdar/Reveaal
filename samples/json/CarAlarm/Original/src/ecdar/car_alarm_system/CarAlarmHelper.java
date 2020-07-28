package ecdar.car_alarm_system;

import ecdar.sut.TestDriver;

import java.time.Instant;

class CarAlarmHelper {

    static TestDriver handler;
    static boolean stepDone;
    private CarAlarm carAlarm;

    CarAlarmHelper(){

    }

    void start() {
        handler = TestDriver.createHandler(100.0, true);
        carAlarm = new CarAlarm();

        carAlarm.clockX = handler.resetTime();
        carAlarm.nextLocation = CarAlarm.location.L0;

        handler.start(this::runStep);
    }

    private void runStep() {
        stepDone = false;

        while (!stepDone) {
            carAlarm.update();
        }

        handler.onStepDone(this::runStep);
    }

    static boolean inputReady() {
        return handler.inputReady();
    }

    static String read() {
        return handler.read();
    }

    static void write(final String... messages) {
        handler.write(messages);
    }

    static double getValue(final Instant clock) {
        return handler.getValue(clock);
    }
}
