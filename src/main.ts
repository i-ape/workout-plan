import { invoke } from '@tauri-apps/api/core';

interface Exercise {
    id: string;
    name: string;
    category: string;
    notes?: string;
}

interface Set {
    reps: number;
    weight: number;
    rpe?: number;
}

interface LoggedExercise {
    exercise: Exercise;
    sets: Set[];
}

interface WorkoutSession {
    id: string;
    date: string;
    exercises: LoggedExercise[];
    notes?: string;
}

async function logSet() {
    const name = (document.getElementById('exercise-name') as HTMLInputElement).value.trim();
    const reps = parseInt((document.getElementById('reps') as HTMLInputElement).value);
    const weight = parseFloat((document.getElementById('weight') as HTMLInputElement).value);

    if (!name) {
        showStatus("Please enter exercise name", "red");
        return;
    }

    const exercise: Exercise = {
        id: "",
        name: name,
        category: "Other"
    };

    const set: Set = { reps, weight };

    try {
        await invoke('log_set', { exercise, set });
        showStatus(`Logged ${reps} × ${weight}kg on ${name}`, "green");
        
        (document.getElementById('exercise-name') as HTMLInputElement).value = '';
        loadCurrentWorkout(); // Refresh current workout
    } catch (err) {
        showStatus("Failed to log set", "red");
    }
}

async function loadCurrentWorkout() {
    try {
        const workout: WorkoutSession | null = await invoke('get_current_workout');
        const container = document.getElementById('current-workout') as HTMLDivElement;
        
        if (!workout || workout.exercises.length === 0) {
            container.innerHTML = '<p>No sets logged today yet.</p>';
            return;
        }

        let html = `<p><strong>${new Date(workout.date).toLocaleDateString()}</strong></p><ul>`;
        
        workout.exercises.forEach(ex => {
            html += 
                `<li>
                    <strong>${ex.exercise.name}</strong><br>`;
                    \( {ex.sets.map(s =>  \){s.reps} × ${s.weight}kg).join(' | ')}
                </li>;
        });
        
        html += '</ul>';
        container.innerHTML = html;
    } catch (err) {
        console.error(err);
    }
}

async function loadHistory() {
    try {
        const history: WorkoutSession[] = await invoke('get_workout_history');
        const container = document.getElementById('history-list') as HTMLDivElement;
        container.innerHTML = '';

        if (history.length === 0) {
            container.innerHTML = '<p>No past workouts yet.</p>';
            return;
        }

        history.forEach(session => {
            const div = document.createElement('div');
            div.className = 'workout-session';
            const date = new Date(session.date).toLocaleDateString();
            div.innerHTML = `<strong>${date}</strong> - ${session.exercises.length} exercises`;
            container.appendChild(div);
        });
    } catch (err) {
        showStatus("Failed to load history", "red");
    }
}

function showStatus(message: string, color: string = "black") {
    const status = document.getElementById('status') as HTMLParagraphElement;
    status.textContent = message;
    status.style.color = color;
}

// Load current workout when app starts
loadCurrentWorkout();

// Event Listeners
document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('load-history')!.addEventListener('click', loadHistory);