<script>
    import { GetExercises, LogSet } from '../wailsjs/go/main/App'; // Wails auto-generates this

    let exercises = [];
    let currentExercise = { name: "", category: "" };
    let currentSet = { reps: 0, weight: 0 };

    async function loadExercises() {
        exercises = await GetExercises();
    }

    async function saveSet() {
        await LogSet(currentExercise, currentSet);
        alert("Set logged!");
    }
</script>

<!-- Simple UI for quick logging -->
<input bind:value={currentExercise.name} placeholder="Exercise name" />
<input type="number" bind:value={currentSet.reps} placeholder="Reps" />
<input type="number" bind:value={currentSet.weight} placeholder="Weight (kg)" />
<button on:click={saveSet}>Log Set</button>

<button on:click={loadExercises}>Load Exercises</button>