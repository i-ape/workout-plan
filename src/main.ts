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

async function generateWarmup() {
    const targetInput = document.getElementById('warmup-target') as HTMLInputElement;
    const barInput = document.getElementById('warmup-bar') as HTMLInputElement;
    const resultEl = document.getElementById('warmup-result') as HTMLDivElement;

    const workingWeight = parseFloat(targetInput.value);
    const barWeight = parseFloat(barInput.value);

    if (isNaN(workingWeight) || isNaN(barWeight)) {
        showStatus("Enter a valid working weight and bar weight", "red");
        return;
    }

    try {
        const sets = await invoke('calc_warmup_sets', { workingWeight, barWeight }) as [number, number][];
        const rows = sets.map(([weight, reps]) => `<li>${weight}kg × ${reps} reps</li>`).join('');

        resultEl.innerHTML = `
            <ul class="space-y-1">${rows}</ul>
            <p class="mt-2 text-sm text-zinc-500">Then: ${workingWeight}kg for your working sets</p>
        `;
    } catch (error) {
        showStatus("Failed to generate warm-up sets", "red");
    }
}

async function logSet() {
    const nameInput = document.getElementById('exercise-name') as HTMLInputElement;
    const categoryInput = document.getElementById('exercise-category') as HTMLSelectElement;
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
        category: categoryInput.value,
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
        loadExerciseDropdown();
        loadCategoryFocus();
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

            const setsHtml = item.sets.map((s, i) => `
                <div class="flex items-center justify-between gap-2 py-1">
                    <span>${s.reps}×${s.weight}kg${s.rpe ? ` @${s.rpe}` : ''}</span>
                    <span class="flex gap-2">
                        <button class="edit-set-btn text-xs px-2 py-1 bg-zinc-700 hover:bg-zinc-600 rounded-lg"
                                data-exercise="${item.exercise.name}" data-index="${i}"
                                data-reps="${s.reps}" data-weight="${s.weight}" data-rpe="${s.rpe ?? ''}">
                            Edit
                        </button>
                        <button class="delete-set-btn text-xs px-2 py-1 bg-red-800 hover:bg-red-700 rounded-lg"
                                data-exercise="${item.exercise.name}" data-index="${i}">
                            Delete
                        </button>
                    </span>
                </div>
            `).join('');

            html +=
                `<li class="workout-session">
                    <strong>${item.exercise.name}</strong>
                    ${setsHtml}
                    <small class="text-emerald-400">
                        1RM: ${oneRM.toFixed(1)}kg | Set volume: ${volume.toFixed(0)}kg | Total volume: ${totalVolume.toFixed(0)}kg${rpeText}
                    </small>
                    ${bestSetText ? `<br><small>${bestSetText}</small>` : ''}
                    ${item.exercise.notes ? `<br><small>${item.exercise.notes}</small>` : ''}
                </li>`;
        }

        html += '</ul>';
        container.innerHTML = html;

        // Wire up edit/delete buttons (re-created every render, so re-attach each time)
        container.querySelectorAll('.delete-set-btn').forEach(btn => {
            btn.addEventListener('click', async (e) => {
                const target = e.currentTarget as HTMLButtonElement;
                const exerciseName = target.dataset.exercise!;
                const index = parseInt(target.dataset.index!);
                await handleDeleteSet(exerciseName, index);
            });
        });

        container.querySelectorAll('.edit-set-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const target = e.currentTarget as HTMLButtonElement;
                openEditSet(
                    target.dataset.exercise!,
                    parseInt(target.dataset.index!),
                    parseInt(target.dataset.reps!),
                    parseFloat(target.dataset.weight!),
                    target.dataset.rpe ? parseFloat(target.dataset.rpe) : undefined
                );
            });
        });
    } catch (error) {
        console.error("Failed to load current workout:", error);
    }
}

async function handleDeleteSet(exerciseName: string, index: number) {
    try {
        await invoke('delete_set', { exerciseName, setIndex: index });
        showStatus(`Deleted set from ${exerciseName}`, "green");
        loadCurrentWorkout();
    } catch (error) {
        showStatus("Failed to delete set", "red");
    }
}

function openEditSet(exerciseName: string, index: number, reps: number, weight: number, rpe?: number) {
    const newReps = prompt(`Reps for ${exerciseName}:`, reps.toString());
    if (newReps === null) return;

    const newWeight = prompt(`Weight (kg) for ${exerciseName}:`, weight.toString());
    if (newWeight === null) return;

    const newRpe = prompt(`RPE (optional, leave blank for none):`, rpe?.toString() ?? '');
    if (newRpe === null) return;

    const set: Set = {
        reps: parseInt(newReps),
        weight: parseFloat(newWeight),
        rpe: newRpe.trim() ? parseFloat(newRpe) : undefined,
    };

    if (isNaN(set.reps) || isNaN(set.weight)) {
        showStatus("Invalid reps or weight", "red");
        return;
    }

    handleEditSet(exerciseName, index, set);
}

async function handleEditSet(exerciseName: string, index: number, set: Set) {
    try {
        await invoke('edit_set', { exerciseName, setIndex: index, set });
        showStatus(`Updated set on ${exerciseName}`, "green");
        loadCurrentWorkout();
    } catch (error) {
        showStatus("Failed to update set", "red");
    }
}

function startRestTimer() {
    const btn = document.getElementById('rest-btn') as HTMLButtonElement;
    const durationInput = document.getElementById('rest-duration') as HTMLInputElement;

    // If a timer is already running, stop it instead of starting a new one
    if (restInterval !== null) {
        clearInterval(restInterval);
        restInterval = null;
        btn.textContent = "Start Rest Timer";
        return;
    }

    const duration = parseInt(durationInput.value);
    if (isNaN(duration) || duration <= 0) {
        showStatus("Enter a valid rest duration", "red");
        return;
    }

    timeLeft = duration;
    btn.textContent = `Rest: ${timeLeft}s`;

    restInterval = window.setInterval(() => {
        timeLeft--;
        btn.textContent = `Rest: ${timeLeft}s`;
        if (timeLeft <= 0) {
            clearInterval(restInterval!);
            restInterval = null;
            btn.textContent = "Start Rest Timer";
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

async function loadExerciseDropdown() {
    try {
        const exercises = await invoke('get_all_exercises') as Exercise[];
        const select = document.getElementById('progress-exercise-select') as HTMLSelectElement;

        select.innerHTML = '<option value="">Select an exercise...</option>';

        // De-duplicate by name, sort alphabetically
        const uniqueNames = Array.from(new Set(exercises.map(e => e.name))).sort();

        for (const name of uniqueNames) {
            const option = document.createElement('option');
            option.value = name;
            option.textContent = name;
            select.appendChild(option);
        }
    } catch (error) {
        console.error("Failed to load exercises:", error);
    }
}

async function loadExerciseProgress() {
    const select = document.getElementById('progress-exercise-select') as HTMLSelectElement;
    const container = document.getElementById('exercise-progress-list') as HTMLDivElement;
    const exerciseName = select.value;

    if (!exerciseName) {
        container.innerHTML = '';
        return;
    }

    try {
        const progress = await invoke('get_exercise_progress', { exerciseName }) as [string, number][];

        if (progress.length === 0) {
            container.innerHTML = '<p>No logged sets for this exercise yet.</p>';
            return;
        }

        const rows = progress
            .map(([date, oneRm]) => `<li>${date}: <strong>${oneRm.toFixed(1)}kg</strong> est. 1RM</li>`)
            .join('');

        container.innerHTML = `<ul class="space-y-1 text-sm text-zinc-300">${rows}</ul>`;
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
            const [, current] = trend[trend.length - 1];
            const [, previous] = trend[trend.length - 2];
            const progress = await invoke('calc_progress_percent', { current, previous }) as number;
            const sign = progress >= 0 ? '+' : '';
            progressText = `<p>Change vs previous week: <strong>${sign}${progress.toFixed(1)}%</strong></p>`;
        }

        const rows = trend.map(([week, vol]) => `<li>${week}: ${vol.toFixed(0)}kg</li>`).join('');

        container.innerHTML = `
            <p>Average weekly volume: <strong>${avgVolume.toFixed(0)}kg</strong></p>
            ${progressText}
            <ul class="mt-2 space-y-1 text-sm text-zinc-400">${rows}</ul>
        `;
    } catch (error) {
        console.error("Failed to load weekly trend:", error);
    }
}

let calendarViewDate = new Date();

async function renderCalendar() {
    const grid = document.getElementById('calendar-grid') as HTMLDivElement;
    const label = document.getElementById('cal-month-label') as HTMLSpanElement;

    const year = calendarViewDate.getFullYear();
    const month = calendarViewDate.getMonth();

    label.textContent = calendarViewDate.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });

    const workoutDates = await invoke('get_workout_dates') as string[];
    const workoutDateSet = new Set(workoutDates);

    const firstOfMonth = new Date(year, month, 1);
    const startWeekday = firstOfMonth.getDay();
    const daysInMonth = new Date(year, month + 1, 0).getDate();

    const dayLabels = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
    let html = dayLabels.map(d => `<div class="text-zinc-500 font-medium">${d}</div>`).join('');

    for (let i = 0; i < startWeekday; i++) {
        html += `<div></div>`;
    }

    for (let day = 1; day <= daysInMonth; day++) {
        const dateObj = new Date(year, month, day);
        const dateStr = dateObj.toISOString().split('T')[0];
        const hasWorkout = workoutDateSet.has(dateStr);
        const isToday = dateStr === new Date().toISOString().split('T')[0];

        const baseClasses = 'py-2 rounded-lg cursor-pointer transition';
        const stateClasses = hasWorkout
            ? 'bg-emerald-600 hover:bg-emerald-500 font-semibold'
            : 'bg-zinc-900 hover:bg-zinc-800';
        const todayRing = isToday ? 'ring-2 ring-orange-400' : '';

        html += `<div class="${baseClasses} ${stateClasses} ${todayRing}" data-date="${dateStr}">${day}</div>`;
    }

    grid.innerHTML = html;

    grid.querySelectorAll('[data-date]').forEach(cell => {
        cell.addEventListener('click', (e) => {
            const date = (e.currentTarget as HTMLElement).dataset.date!;
            showCalendarDay(date);
        });
    });
}

async function loadCategoryFocus() {
    try {
        const volumes = await invoke('get_category_volume') as [string, number][];
        const lastTrained = await invoke('get_last_trained_by_category') as [string, string][];
        const container = document.getElementById('category-focus') as HTMLDivElement;

        if (volumes.length === 0) {
            container.innerHTML = '<p>No workout history yet.</p>';
            return;
        }

        const lastTrainedMap = new Map(lastTrained);
        const today = new Date().toISOString().split('T')[0];

        const rows = volumes.map(([category, volume]) => {
            const lastDate = lastTrainedMap.get(category);
            let daysAgoText = 'never';
            let staleClass = '';

            if (lastDate) {
                const daysAgo = Math.floor(
                    (new Date(today).getTime() - new Date(lastDate).getTime()) / (1000 * 60 * 60 * 24)
                );
                daysAgoText = daysAgo === 0 ? 'today' : `${daysAgo}d ago`;
                if (daysAgo >= 7) staleClass = 'text-orange-400';
            }

            return `
                <div class="flex justify-between py-2 border-b border-zinc-800">
                    <span class="font-medium">${category}</span>
                    <span class="text-zinc-400">${volume.toFixed(0)}kg total</span>
                    <span class="${staleClass}">${daysAgoText}</span>
                </div>
            `;
        }).join('');

        container.innerHTML = rows;
    } catch (error) {
        console.error("Failed to load category focus:", error);
    }
}

let knownExercises: Exercise[] = [];

async function loadExerciseSuggestions() {
    try {
        knownExercises = await invoke('get_all_exercises') as Exercise[];
        const datalist = document.getElementById('exercise-suggestions') as HTMLDataListElement;

        const uniqueNames = Array.from(new Set(knownExercises.map(e => e.name))).sort();
        datalist.innerHTML = uniqueNames.map(name => `<option value="${name}"></option>`).join('');
    } catch (error) {
        console.error("Failed to load exercise suggestions:", error);
    }
}

function autoFillCategory() {
    const nameInput = document.getElementById('exercise-name') as HTMLInputElement;
    const categoryInput = document.getElementById('exercise-category') as HTMLSelectElement;

    const match = knownExercises.find(
        e => e.name.toLowerCase() === nameInput.value.trim().toLowerCase()
    );

    if (match) {
        categoryInput.value = match.category;
    }
}

async function showCalendarDay(date: string) {
    const detail = document.getElementById('calendar-day-detail') as HTMLDivElement;
    try {
        const workout = await invoke('get_workout_by_date', { date }) as WorkoutSession | null;

        if (!workout || workout.exercises.length === 0) {
            detail.innerHTML = `<p><strong>${date}</strong> — no sets logged.</p>`;
            return;
        }

        const rows = workout.exercises.map(item => {
            const sets = item.sets.map(s => `${s.reps}×${s.weight}kg${s.rpe ? ` @${s.rpe}` : ''}`).join(' | ');
            return `<li><strong>${item.exercise.name}</strong>: ${sets}</li>`;
        }).join('');

        detail.innerHTML = `<p class="mb-2"><strong>${date}</strong></p><ul class="space-y-1">${rows}</ul>`;
    } catch (error) {
        detail.innerHTML = `<p>Failed to load workout for ${date}.</p>`;
    }
}

function changeCalendarMonth(offset: number) {
    calendarViewDate = new Date(calendarViewDate.getFullYear(), calendarViewDate.getMonth() + offset, 1);
    renderCalendar();
}

async function exportToCsv() {
    try {
        const csv = await invoke('export_to_csv') as string;
        const blob = new Blob([csv], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.href = url;
        a.download = `workout-data-${new Date().toISOString().split('T')[0]}.csv`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        showStatus("✅ Exported to CSV", "green");
    } catch (error) {
        showStatus("Failed to export CSV", "red");
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
loadExerciseDropdown();
renderCalendar();
loadCategoryFocus();

document.getElementById('log-btn')!.addEventListener('click', logSet);
document.getElementById('rest-btn')!.addEventListener('click', startRestTimer);
document.getElementById('load-history')!.addEventListener('click', loadHistory);
document.getElementById('load-prs')!.addEventListener('click', loadPersonalRecords);
document.getElementById('calc-1rm-btn')!.addEventListener('click', calculate1RM);
document.getElementById('suggest-weight-btn')!.addEventListener('click', calculateSuggestedWeight);
document.getElementById('progress-exercise-select')!.addEventListener('change', loadExerciseProgress);
document.getElementById('prev-month')!.addEventListener('click', () => changeCalendarMonth(-1));
document.getElementById('next-month')!.addEventListener('click', () => changeCalendarMonth(1));
document.getElementById('export-csv-btn')!.addEventListener('click', exportToCsv);
document.getElementById('warmup-btn')!.addEventListener('click', generateWarmup);
document.getElementById('exercise-name')!.addEventListener('input', autoFillCategory);
loadExerciseSuggestions();