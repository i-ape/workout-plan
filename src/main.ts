import { invoke } from '@tauri-apps/api/core';

// Data Types
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

interface WorkoutSession {
    id: string;
    date: string;
    exercises: Array<{
        exercise: Exercise;
        sets: Set[];
    }>;
}

// Rest Timer
let restInterval: number | null = null;
let timeLeft = 90;

async function logSet() {
    const nameInput = document.getElementById('exercise-name') as HTMLInputElement;
    const repsInput = document.getElementById('reps') as HTMLInputElement;
    const weightInput = document.getElementById('weight') as HTMLInputElement;
    const rpeInput = document.getElementById('rpe') as HTMLInputElement;
    const notesInput = document.getElementById('exercise-notes') as HTMLInputElement;

    const name = nameInput.value.trim();

    if (!name) {
        showStatus("Please enter an exercise name", "red");
        return;
    }

    const notes = notesInput.value.trim();
    const rpeValue = rpeInput.value.trim();

    const exercise: Exercise = {
        id: "",
        name,
        category: "Other",
        notes: notes || undefined,
    };
    const set: Set = {
        reps: parseInt(repsInput.value),
        weight: parseFloat(weightInput.value),
        rpe: rpeValue ? parseFloat(rpeValue) : undefined,
    };

    try {
        await invoke('log_set', { exercise, set });
        showStatus(`✅ Logged ${set.reps} × ${set.weight}kg on ${name}`, "green");

        nameInput.value = '';
        rpeInput.value = '';
        notesInput.value = '';
        loadCurrentWorkout();
    } catch (error) {
        showStatus(`❌ Failed to log set`, "red");
    }
}

async function loadCurrentWorkout() {
    try {
        const workout = await invoke('get_current_workout') as WorkoutSession | null;
        const container = document.getElementById('current-workout') as HTMLDivElement;

        if (!workout || workout.exercises.length === 0) {
            container.innerHTML = '<p>No sets logged today yet.</p>';
            return;
        }

        let html = `<p><strong>Today - ${new Date(workout.date).toLocaleDateString()}</strong></p><ul>`;

        for (const item of workout.exercises) {
            const lastSet = item.sets[item.sets.length - 1];
            const oneRM = await invoke('calc_1rm', { weight: lastSet.weight, reps: lastSet.reps }) as number;
            const volume = await invoke('calc_volume', { weight: lastSet.weight, reps: lastSet.reps }) as number;
            const totalVolume = await invoke('calc_total_volume', { sets: item.sets }) as number;
            const bestSet = await invoke('find_best_set', { sets: item.sets }) as Set | null;
            const rpeText = lastSet.rpe !== undefined && lastSet.rpe !== null ? ` @RPE ${lastSet.rpe}` : '';
            const bestSetText = bestSet ? `Best set today: ${bestSet.reps}×${bestSet.weight}kg` : '';

            html +=
                `<li class="workout-session">
                    <strong>${item.exercise.name}</strong><br>
                    ${item.sets.map(s => `${s.reps}×${s.weight}kg${s.rpe ? ` @${s.rpe}` : ''}`).join(' | ')}
                    <br>
                    <small class="text-emerald-400">
                        1RM: ${oneRM.toFixed(1)}kg | Set volume: ${volume.toFixed(0)}kg | Total volume: ${totalVolume.toFixed(0)}kg${rpeText}
                    </small>
                    ${bestSetText ? `<br><small>${bestSetText}</small>` : ''}
                    ${item.exercise.notes ? `<br><small>${item.exercise.notes}</small>` : ''}
                </li>`;
        }

        html += '</ul>';
        container.innerHTML = html;
    } catch (error) {
        console.error("Failed to load current workout:", error);
    }
}

function startRestTimer() {
    if (restInterval) clearInterval(restInterval);

    timeLeft = 90;
    const btn = document.getElementById('rest-btn') as HTMLButtonElement;
    btn.textContent = `Rest: ${timeLeft}s`;

    restInterval = setInterval(() => {
        timeLeft--;
        btn.textContent = `Rest: ${timeLeft}s`;
        if (timeLeft <= 0) {
            clearInterval(restInterval!);
            btn.textContent = "Start Rest Timer (90s)";
            showStatus("✅ Rest finished!", "green");
        }
    }, 1000);
}

async function loadHistory() {
    try {
        const history = await invoke('get_workout_history') as WorkoutSession[];
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
            div.innerHTML = `<strong>${date}</strong> — ${session.exercises.length} exercises`;
            container.appendChild(div);
        });
    } catch (error) {
        showStatus(`Failed to load history`, "red");
    }
}

async function loadPersonalRecords() {
    try {
        const records = await invoke('get_personal_records') as [string, Set][];
        const container = document.getElementById('pr-list') as HTMLDivElement;
        container.innerHTML = '';

        if (records.length === 0) {
            container.innerHTML = '<p>No personal records yet — log some sets first.</p>';
            return;
        }

        // Sort alphabetically by exercise name for a stable, readable list
        records.sort((a, b) => a[0].localeCompare(b[0]));

        let html = '<ul>';
        for (const [name, set] of records) {
            const rpeText = set.rpe !== undefined && set.rpe !== null ? ` @RPE ${set.rpe}` : '';
            html += `<li class="workout-session">
                <strong>${name}</strong><br>
                Best set: ${set.reps} × ${set.weight}kg${rpeText}
            </li>`;
        }
        html += '</ul>';
        container.innerHTML = html;
    } catch (error) {
        showStatus(`Failed to load personal records`, "red");
    }
}

async function calculate1RM() {
    const weightInput = document.getElementById('calc-weight') as HTMLInputElement;
    const repsInput = document.getElementById('calc-reps') as HTMLInputElement;
    const resultEl = document.getElementById('calc-1rm-result') as HTMLDivElement;

    const weight = parseFloat(weightInput.value);
    const reps = parseInt(repsInput.value);

    if (isNaN(weight) || isNaN(reps)) {
        showStatus("Enter a valid weight and reps", "red");
        return;
    }

    try {
        const epley = await invoke('calc_1rm', { weight, reps }) as number;
        const brzycki = await invoke('calc_1rm_brzycki', { weight, reps }) as number;
        const trainingMax = await invoke('calc_training_max', { oneRm: epley }) as number;

        resultEl.innerHTML = `
            <p>Epley 1RM: <strong>${epley.toFixed(1)}kg</strong></p>
            <p>Brzycki 1RM: <strong>${brzycki.toFixed(1)}kg</strong></p>
            <p>Training Max (90% Epley): <strong>${trainingMax.toFixed(1)}kg</strong></p>
        `;
    } catch (error) {
        showStatus("Failed to calculate 1RM", "red");
    }
}

async function calculateSuggestedWeight() {
    const oneRmInput = document.getElementById('suggest-1rm') as HTMLInputElement;
    const repsInput = document.getElementById('suggest-reps') as HTMLInputElement;
    const rpeInput = document.getElementById('suggest-rpe') as HTMLInputElement;
    const resultEl = document.getElementById('suggest-result') as HTMLDivElement;

    const oneRm = parseFloat(oneRmInput.value);
    const reps = parseInt(repsInput.value);
    const targetRpe = parseFloat(rpeInput.value);

    if (isNaN(oneRm) || isNaN(reps) || isNaN(targetRpe)) {
        showStatus("Enter a valid 1RM, reps, and target RPE", "red");
        return;
    }

    try {
        const suggested = await invoke('suggest_weight_for_rpe', { oneRm, reps, targetRpe }) as number;
        resultEl.innerHTML = `<p>Suggested weight: <strong>${suggested.toFixed(1)}kg</strong></p>`;
    } catch (error) {
        showStatus("Failed to calculate suggested weight", "red");
    }
}

async function loadExerciseProgress() {
    const nameInput = document.getElementById('progress-exercise-name') as HTMLInputElement;
    const container = document.getElementById('exercise-progress-result') as HTMLDivElement;

    const name = nameInput.value.trim();
    if (!name) {
        showStatus("Enter an exercise name to view progress", "red");
        return;
    }

    try {
        const progress = await invoke('get_exercise_progress', { exerciseName: name }) as [string, number][];

        if (progress.length === 0) {
            container.innerHTML = '<p>No history for this exercise yet.</p>';
            return;
        }

        const rows = progress.map(([date, oneRm]) =>
            `<div class="flex justify-between"><span>${date}</span><span>${oneRm.toFixed(1)}kg est. 1RM</span></div>`
        ).join('');

        let trendText = '';
        if (progress.length >= 2) {
            const first = progress[0][1];
            const latest = progress[progress.length - 1][1];
            const change = await invoke('calc_progress_percent', { current: latest, previous: first }) as number;
            const sign = change >= 0 ? '+' : '';
            trendText = `<p class="mt-2 text-emerald-400">Overall change: ${sign}${change.toFixed(1)}%</p>`;
        }

        container.innerHTML = `<div class="space-y-1 text-sm text-zinc-400">${rows}</div>${trendText}`;
    } catch (error) {
        showStatus("Failed to load exercise progress", "red");
    }
}

async function loadWeeklyTrend() {
    try {
        const trend = await invoke('get_weekly_volume_trend') as [string, number][];
        const container = document.getElementById('weekly-trend') as HTMLDivElement;

        if (trend.length === 0) {
            container.innerHTML = '<p>No workout history yet.</p>';
            return;
        }

        const volumes = trend.map(([, v]) => v);
        const avgVolume = await invoke('calc_weekly_volume', { volumes }) as number;

        let progressText = '';
        if (trend.length >= 2) {
            const current = trend[trend.length - 1][1];
            const previous = trend[trend.length - 2][1];
            const progress = await invoke('calc_progress_percent', { current, previous }) as number;
            const sign = progress >= 0 ? '+' : '';
            progressText = `<p>Change vs previous week: <strong>${sign}${progress.toFixed(1)}%</strong></p>`;
        }

        const weekRows = trend.map(([week, volume]) =>
            `<div class="flex justify-between"><span>${week}</span><span>${volume.toFixed(0)}kg</span></div>`
        ).join('');

        container.innerHTML = `
            <p>Average weekly volume: <strong>${avgVolume.toFixed(0)}kg</strong></p>
            ${progressText}
            <div class="mt-3 space-y-1 text-sm text-zinc-400">${weekRows}</div>
        `;
    } catch (error) {
        console.error("Failed to load weekly trend:", error);
    }
}

function showStatus(message: string, color: string = "white") {
    const statusEl = document.getElementById('status') as HTMLParagraphElement;
    statusEl.textContent = message;
    statusEl.style.color = color;
}

// Initialize
loadCurrentWorkout();
loadWeeklyTrend();

document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('rest-btn')!.addEventListener('click', startRestTimer);
document.getElementById('load-history')!.addEventListener('click', loadHistory);
document.getElementById('load-prs')!.addEventListener('click', loadPersonalRecords);
document.getElementById('calc-1rm-btn')!.addEventListener('click', calculate1RM);
document.getElementById('suggest-weight-btn')!.addEventListener('click', calculateSuggestedWeight);
document.getElementById('load-progress-btn')!.addEventListener('click', loadExerciseProgress);