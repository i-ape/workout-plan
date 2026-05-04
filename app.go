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
	repo, err := repository.NewRepository()
	if err != nil {
		// handle error (for now just panic or log)
		panic(err)
	}
	return &App{repo: repo}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

// Example methods you can call from Svelte
func (a *App) GetExercises() ([]models.Exercise, error) {
	return a.repo.GetAllExercises()
}

func (a *App) LogSet(ex models.Exercise, s models.Set) error {
	return a.repo.LogSet(ex, s)
}
