package models

import "time"

type Exercise struct {
    ID       string json:"id"
    Name     string json:"name"
    Category string json:"category" // Push, Pull, Legs, etc.
    Notes    string json:"notes,omitempty"
}

type Set struct {
    Reps   int     json:"reps"
    Weight float64 json:"weight"
    RPE    float64 json:"rpe,omitempty"
}

type WorkoutSession struct {
    ID        string    json:"id"
    Date      time.Time json:"date"
    Exercises []LoggedExercise json:"exercises"
    Notes     string    json:"notes,omitempty"
}

type LoggedExercise struct {
    Exercise Exercise json:"exercise"
    Sets     []Set    json:"sets"
}