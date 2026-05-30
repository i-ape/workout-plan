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
        showStatus(Logged ${reps} × ${weight}kg on ${name}, "green");
        
        // Clear input for next set
        (document.getElementById('exercise-name') as HTMLInputElement).value = '';
    } catch (err) {
        showStatus("Failed to log set", "red");
        console.error(err);
    }
}

async function loadHistory() {
    try {
        const history: WorkoutSession[] = await invoke('get_workout_history');
        const container = document.getElementById('history-list') as HTMLDivElement;
        container.innerHTML = '';

        if (history.length === 0) {
            container.innerHTML = '<p>No workouts yet. Start logging sets!</p>';
            return;
        }

        history.forEach(session => {
            const date = new Date(session.date).toLocaleDateString();
            const div = document.createElement('div');
            div.className = 'workout-session';
            div.innerHTML = 
                <strong>${date}</strong>
                <ul>
                    ${session.exercises.map(ex => 
                        <li>
                            ${ex.exercise.name} — 
                            \( {ex.sets.map(s =>  \){s.reps}×${s.weight}kg).join(', ')}
                        </li>
                    ).join('')}
                </ul>
            ;
            container.appendChild(div);
        });
    } catch (err) {
        showStatus("Failed to load history", "red");
        console.error(err);
    }
}

function showStatus(message: string, color: string = "black") {
    const status = document.getElementById('status') as HTMLParagraphElement;
    status.textContent = message;
    status.style.color = color;
}

// Event Listeners
document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('load-history')!.addEventListener('click', loadHistory);