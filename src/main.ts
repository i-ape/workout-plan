import { invoke } from '@tauri-apps/api/core';

// ==================== DATA TYPES ====================
interface Exercise {
    id: string;
    name: string;
    category: string;
}

interface Set {
    reps: number;
    weight: number;
}

interface WorkoutSession {
    id: string;
    date: string;
    exercises: Array<{
        exercise: Exercise;
        sets: Set[];
    }>;
}

// ==================== MAIN FUNCTIONS ====================

async function logSet() {
    const nameInput = document.getElementById('exercise-name') as HTMLInputElement;
    const repsInput = document.getElementById('reps') as HTMLInputElement;
    const weightInput = document.getElementById('weight') as HTMLInputElement;

    const name = nameInput.value.trim();

    if (!name) {
        showStatus("Please enter an exercise name", "red");
        return;
    }

    const exercise: Exercise = {
        id: "",
        name: name,
        category: "Other"
    };

    const set: Set = {
        reps: parseInt(repsInput.value),
        weight: parseFloat(weightInput.value)
    };

    try {
        await invoke('log_set', { exercise, set });
        showStatus(✅ Logged ${set.reps} × ${set.weight}kg on ${name}, "green");
        
        nameInput.value = '';
        loadCurrentWorkout();
    } catch (error) {
        showStatus("❌ Failed to log set", "red");
        console.error(error);
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

        let html = <p><strong>Today - ${new Date(workout.date).toLocaleDateString()}</strong></p><ul>;

        workout.exercises.forEach(item => {
            html += 
                <li>
                    <strong>${item.exercise.name}</strong><br>
                    \( {item.sets.map(s =>  \){s.reps} × ${s.weight}kg).join(' | ')}
                </li>;
        });

        html += '</ul>';
        container.innerHTML = html;
    } catch (error) {
        console.error("Failed to load current workout:", error);
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
            div.innerHTML = <strong>${date}</strong> — ${session.exercises.length} exercises;
            container.appendChild(div);
        });
    } catch (error) {
        showStatus("Failed to load history", "red");
    }
}

function showStatus(message: string, color: string = "black") {
    const statusEl = document.getElementById('status') as HTMLParagraphElement;
    statusEl.textContent = message;
    statusEl.style.color = color;
}

// ==================== INITIALIZE ====================
loadCurrentWorkout();

document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('load-history')!.addEventListener('click', loadHistory);