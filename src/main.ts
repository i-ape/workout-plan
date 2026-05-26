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

async function logSet() {
    const name = (document.getElementById('exercise-name') as HTMLInputElement).value;
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
        showStatus(Logged ${reps} × ${weight}kg on ${name}!, "green");
    } catch (err) {
        showStatus("Failed to log set", "red");
        console.error(err);
    }
}

async function loadExercises() {
    try {
        const exercises: Exercise[] = await invoke('get_all_exercises');
        const list = document.getElementById('exercise-list') as HTMLUListElement;
        list.innerHTML = '';

        exercises.forEach(ex => {
            const li = document.createElement('li');
            li.textContent = \( {ex.name} ( \){ex.category});
            list.appendChild(li);
        });
    } catch (err) {
        showStatus("Failed to load exercises", "red");
    }
}

function showStatus(message: string, color: string) {
    const status = document.getElementById('status') as HTMLParagraphElement;
    status.textContent = message;
    status.style.color = color;
}

// Event listeners
document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('load-exercises')!.addEventListener('click', loadExercises);