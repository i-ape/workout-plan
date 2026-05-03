package repository

import (
    "database/sql"
    _ "modernc.org/sqlite"
    "exercise-tracker/internal/models" // adjust if your module name is different
    "os"
    "path/filepath"
)

type Repository struct {
    db *sql.DB
}

func NewRepository() (*Repository, error) {
    // Store DB in a user-friendly location
    home, _ := os.UserHomeDir()
    dbDir := filepath.Join(home, ".exercise-tracker")
    os.MkdirAll(dbDir, 0755)

    dbPath := filepath.Join(dbDir, "data.db")

    db, err := sql.Open("sqlite", dbPath)
    if err != nil {
        return nil, err
    }

    // Create tables
    _, err = db.Exec(
        CREATE TABLE IF NOT EXISTS exercises (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS workouts (
            id TEXT PRIMARY KEY,
            date DATETIME NOT NULL,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS workout_sets (
            id TEXT PRIMARY KEY,
            workout_id TEXT,
            exercise_id TEXT,
            reps INTEGER,
            weight REAL,
            rpe REAL,
            order_num INTEGER,
            FOREIGN KEY(workout_id) REFERENCES workouts(id),
            FOREIGN KEY(exercise_id) REFERENCES exercises(id)
        );
    )
    if err != nil {
        return nil, err
    }

    return &Repository{db: db}, nil
}

// Example methods
func (r *Repository) CreateExercise(ex *models.Exercise) error {
    // implementation...
    return nil
}

func (r *Repository) LogWorkout(workout *models.WorkoutSession) error {
    // implementation...
    return nil
}