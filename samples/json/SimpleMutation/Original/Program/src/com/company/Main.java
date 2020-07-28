package com.company;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.time.Duration;
import java.time.Instant;

public class Main {
    public static void main(String[] args) {
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        Instant x;
        String input;

        System.out.println("a");
        x = Instant.now();
        try {
            input = reader.readLine();
            if (input.equals("c") && Duration.between(x, Instant.now()).toMillis() >= 3000 && Duration.between(x, Instant.now()).toMillis() <= 6000) {
                if (Duration.between(x, Instant.now()).toMillis() <= 8000) {
                    System.out.println("a");

                }
            }
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}