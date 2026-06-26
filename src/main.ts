import { invoke } from '@tauri-apps/api/core';

// Data Types
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

// Variables for rest timer
let restInterval: number | null = null;
let timeLeft = 90;

async function logSet() {
    const nameInput = document.getElementById('exercise-name') as HTMLInputElement;
    const repsInput = document.getElementById('reps') as HTMLInputElement;
    const weightInput = document.getElementById('weight') as HTMLInputElement;

    const name = nameInput.value.trim();

    if (!name) {
        showStatus("Please enter an exercise name", "red");
        return;
    }

    const exercise: Exercise = { id: "", name, category: "Other" };
    const set: Set = {
        reps: parseInt(repsInput.value),
        weight: parseFloat(weightInput.value)
    };

    try {
        await invoke('log_set', { exercise, set });
        showStatus(`✅ Logged ${set.reps} × ${set.weight}kg on ${name}`, "green");
        
        nameInput.value = '';
        loadCurrentWorkout();
    } catch (error) {
        showStatus("❌ Failed to log set", "red");
    }
}

async function loadCurrentWorkout() {
    try {
        const workout = await invoke('get_current_workout') as WorkoutSession;
        const container = document.getElementById('current-workout') as HTMLDivElement;

        if (!workout || workout.exercises.length === 0) {
            container.innerHTML = '<p>No sets logged today yet.</p>';
            return;
        }

        let html = `<p><strong>Today - ${new Date(workout.date).toLocaleDateString()}</strong></p><ul>`;

        for (const item of workout.exercises) {
            const lastSet = item.sets[item.sets.length - 1];
            const oneRM = await invoke<number>('calculate_1rm', { weight: lastSet.weight, reps: lastSet.reps });

            html += `
                <li class="workout-session">
                    <strong>${item.exercise.name}</strong><br>
                    ${item.sets.map(s => `${s.reps}×${s.weight}kg`).join(' | ')}
                    <br>
                    <small class="text-emerald-400">Est. 1RM: ${oneRM.toFixed(1)}kg</small>
                </li>
            `;
        }

        html += '</ul>';
        container.innerHTML = html;
    } catch (error) {
        console.error(error);
    }
}

function loadHistory() {
    showStatus('Load history not implemented yet.', 'black');
}

// Rest Timer
function startRestTimer() {
    if (restInterval) clearInterval(restInterval);
    
    timeLeft = 90;
    const btn = document.getElementById('rest-btn') as HTMLButtonElement;
    
    restInterval = setInterval(() => {
        timeLeft--;
        btn.textContent = `Rest: ${timeLeft}s`;
        
        if (timeLeft <= 0) {
            clearInterval(restInterval!);
            btn.textContent = "Start Rest Timer (90s)";
            showStatus("Rest time over! Next set ready", "green");
        }
    }, 1000);
}

// Show status
function showStatus(message: string, color: string = "black") {
    const statusEl = document.getElementById('status') as HTMLParagraphElement;
    statusEl.textContent = message;
    statusEl.style.color = color;
}

// Initialize
loadCurrentWorkout();

document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('rest-btn')!.addEventListener('click', startRestTimer);
document.getElementById('load-history')!.addEventListener('click', loadHistory); // assume you have loadHistory too