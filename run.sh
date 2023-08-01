#!/bin/bash

java -jar ../jatyc/dist/checker/checker.jar -classpath ../jatyc/dist/jatyc.jar -processor jatyc.JavaTypestateChecker *.java
