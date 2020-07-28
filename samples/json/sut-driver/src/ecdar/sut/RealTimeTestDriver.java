package ecdar.sut;

import java.time.Duration;
import java.time.Instant;
import java.util.Timer;
import java.util.TimerTask;

/**
 * Driver for testing using real-time.
 */
public class RealTimeTestDriver extends TestDriver {
    /**
     * The amount of a time unit in ms.
     */
    private final double timeUnit;

    /**
     * Constructs this.
     * @param timeUnit what an Ecdar model time unit corresponds to in ms
     */
    @SuppressWarnings("WeakerAccess")
    public RealTimeTestDriver(double timeUnit) {
        this.timeUnit = timeUnit;
    }

    @Override
    public void start(Runnable stepStarter) {
        stepStarter.run();
    }

    @Override
    public void onStepDone(Runnable startNewStep) {
        new Timer().schedule(new TimerTask() {
            @Override
            public void run() {
                startNewStep.run();
            }
        }, (long) timeUnit / 4);
    }

    @Override
    public double getValue(Instant clock) {
        return Duration.between(clock, Instant.now()).toMillis() / timeUnit;
    }

    @Override
    public Instant resetTime() {
        return Instant.now();
    }
}
