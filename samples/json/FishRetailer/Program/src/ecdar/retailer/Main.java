package ecdar.retailer;

import ecdar.sut.TestDriver;

import java.time.Instant;

public class Main {
    private static final String COIN = "coin";
    private static final String GARNISH = "garnish";
    private static final String TUNA = "tuna";

    private static Instant x;
    private static int loc;
    private static int free;
    private static boolean stepDone;
    private static TestDriver handler;


    public static void main(String[] args) {
        handler = TestDriver.createHandler(200.0, false);

        loc = 0;
        x = handler.resetTime();
        free = 0;

        handler.start(Main::runStep);
    }

    private static void runStep() {
        stepDone = false;

        while (!stepDone) {
            update();
        }

        handler.onStepDone(Main::runStep);
    }

    private static void update() {
        switch (loc) {
            case 0:
                if (handler.inputReady()) {
                    if (handler.read().equals(COIN)) {
                        free = 1;
                        x = handler.resetTime();
                        loc = 1;
                    } else loc = -1;
                } else if (handler.getValue(x) < 3.0 && free == 1) {
                    handler.write(GARNISH);
                    free = 0;
                } else stepDone = true;

                break;
            case 1:
                if (handler.inputReady()) loc = -1;
                else if (handler.getValue(x) > 1.0) {
                    handler.write(TUNA);
                    loc = 0;
                } else stepDone = true;

                break;
            default:
                new RuntimeException("Loc is " + loc).printStackTrace();
                System.exit(1);
        }
    }
}
