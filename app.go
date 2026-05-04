package main

import (
	"context"
	"exercise-tracker/internal/models"
	"exercise-tracker/internal/repository"
)

type App struct {
	ctx  context.Context
	repo *repository.Repository
}

func NewApp() *App {
	repo, _ := repository.NewRepository() // handle error properly in real code
	return &App{repo: repo}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

// Exposed to frontend
func (a *App) LogSet(exercise models.Exercise, set models.Set) error {
	// For simplicity, create a quick workout or expand later
	return nil
}

func (a *App) GetExercises() ([]models.Exercise, error) {
	// Query and return
	return nil, nil
}
