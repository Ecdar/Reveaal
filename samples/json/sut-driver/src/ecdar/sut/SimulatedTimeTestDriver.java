package ecdar.sut;

import java.time.Duration;
import java.time.Instant;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Driver for testing using simulated time.
 * It uses an instant as the time.
 * For every time unit to pass, the handler increments the instant with a millisecond.
 */
public class SimulatedTimeTestDriver extends TestDriver {
    private Instant delayStartTime; // The time when this started delaying
    private Instant time; // The current time
    private Instant maxTime; // The maximum allowed time to delay to, before calling Ecdar

    /**
     * Constructs this.
     */
    @SuppressWarnings("WeakerAccess")
    public SimulatedTimeTestDriver() {
        time = Instant.now();
    }

    @Override
    public void start(final Runnable stepStarter) {
        waitForDelay(stepStarter);
    }

    @Override
    public void onStepDone(final Runnable startNewStep) {

        // If we have outputted or reached the maximum allowed time to delay to, call Ecdar
        // Otherwise, start a new step
        if (hasOutputted || !time.isBefore(maxTime)) {
            write("Delayed: " + Duration.between(delayStartTime, time).toMillis());
            hasOutputted = false;
            waitForDelay(startNewStep);
        } else {
            time = time.plus(Duration.ofMillis(1));
            startNewStep.run();
        }
    }

    private synchronized void waitForDelay(final Runnable startNewStep) {
        if (inputsBuffer.stream().noneMatch(this::isDelay)) {
            tempLinesListeners.add(() -> waitForDelay(startNewStep));
            return;
        }

        // Fetch delay
        String delayLine = inputsBuffer.stream().filter(this::isDelay).findFirst().orElseThrow(() -> new RuntimeException("lines had no delays"));

        final Matcher matcher = Pattern.compile("Delay: (\\d+)").matcher(delayLine);

        if (matcher.find()) { // Simulate delay
            delayStartTime = time;
            maxTime = time.plus(Duration.ofMillis(Long.parseLong(matcher.group(1))));
            inputsBuffer.removeIf(this::isDelay);
        } else throw new RuntimeException("Delay line is not a delay");

        startNewStep.run();
    }

    private boolean isDelay(final String input) {
        final Matcher matcher = Pattern.compile("Delay: (\\d+)").matcher(input);

        return matcher.find();
    }

    @Override
    public double getValue(Instant clock) {
        return Duration.between(clock, time).toMillis();
    }

    @Override
    public Instant resetTime() {
        return time;
    }
}
