package ecdar.sut;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;

/**
 * Driver for running a test with Ecdar.
 * This handles inputs, outputs, and delays.
 * This driver handles a system under test in steps.
 * Call the start method to start the driver.
 * When a step is done (the system needs to delay in order to proceed), call onStepDone.
 */
public abstract class TestDriver {
    boolean hasOutputted = false; // True iff this has outputted since last reset. This is used when simulating time

    final List<String> inputsBuffer = new ArrayList<>();

    final List<Runnable> tempLinesListeners = new ArrayList<>(); // To be called a single time when a line appears

    /**
     * Creates a driver.
     * @param timeUnit what an Ecdar model time unit corresponds to in ms (only used for real-time testing)
     * @param shouldSimulate true iff we should simulated time when testing rather than testing using real-time
     * @return the test driver
     */
    @SuppressWarnings("unused")
    public static TestDriver createHandler(final double timeUnit, final boolean shouldSimulate) {
        if (shouldSimulate) return new SimulatedTimeTestDriver();
        else return new RealTimeTestDriver(timeUnit);
    }

    /**
     * Constructs the driver.
     * Starts reading of inputs from Ecdar.
     */
    TestDriver() {
        new Thread(() -> {
            String line;
            final BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
            try {
                while ((line = reader.readLine()) != null) {
                    final List<Runnable> listeners;

                    synchronized (this) {
                        inputsBuffer.add(line);

                        listeners = new ArrayList<>(tempLinesListeners);
                        tempLinesListeners.clear();
                    }

                    listeners.forEach(Runnable::run);
                }
            } catch (IOException e) {
                e.printStackTrace();
            }
        }).start();
    }

    /**
     * Gets if an input is ready to be read.
     * @return true iff an input is ready
     */
    @SuppressWarnings("unused")
    public boolean inputReady() {
        return !inputsBuffer.isEmpty();
    }

    /**
     * Reads and consumes an input from Ecdar.
     * @return the input
     */
    @SuppressWarnings("unused")
    public String read() {
        final String line = inputsBuffer.get(0);
        inputsBuffer.remove(0);
        return line;
    }

    /**
     * Starts the driver.
     * @param stepStarter the
     */
    @SuppressWarnings("unused")
    public abstract void start(final Runnable stepStarter);

    /**
     * Delays for some time.
     * @param startNewStep the the step to run, once we have delayed
     */
    @SuppressWarnings("unused")
    public abstract void onStepDone(final Runnable startNewStep);

    /**
     * Gets the value of a clock in time units.
     * @param clock the clock
     * @return the value in time units
     */
    @SuppressWarnings("unused")
    public abstract double getValue(final Instant clock);

    /**
     * Gets the clock value for a reset time.
     * @return the value
     */
    @SuppressWarnings("unused")
    public abstract Instant resetTime();


    /**
     * Writes to Ecdar over standard output.
     * @param message the message to write. This should correspond with the model (without !)
     */
    @SuppressWarnings("WeakerAccess")
    public void write(String message) {
        System.out.println(message);
        hasOutputted = true;
    }

    /**
     * Writes some messages to Ecdar over standard output.
     * @param messages the messages to write. This should correspond with the model (without !)
     */
    @SuppressWarnings("unused")
    public void write(final String[] messages) {
        for (String message : messages) {
            write(message);
        }
    }
}
