<script lang="ts">
import { onMount } from 'svelte';
import {fetchAPI} from "../api/api";
import type  {User} from "../state/index";
let user: User = {}
// Get user information
onMount(async () => {
    try {
        const resp = await fetchAPI(`/user`)
        user = resp
    }catch(err){
        throw new Error(err)
    }
})

</script>

<main>
<h1>Gittoken</h1>
{user}
</main>