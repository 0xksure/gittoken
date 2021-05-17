<script lang="ts">
import { onMount } from 'svelte';
import {fetchAPI} from "../api/api";
import type {User} from "../state/index";
let user: User;
console.log("user: ",user)
// Get user information
onMount(async () => {
    try {
        const resp = await fetchAPI(`/user`)
        user = resp
        console.log(resp)
    }catch(err){
        throw new Error(err)
    }
})

async function PostStatus(){
     try {
        const resp = await fetchAPI(`/github/app/post/status`)
        user = resp
        console.log("Status: ",resp)
    }catch(err){
        throw new Error(err)
    }
}

</script>

<main>
<h1>Open source token</h1>
<h3>Github profile {user?.username}</h3>
<form>
<label>
    Add wallet address
<input
    placeholder="wallet address"
/>
</label>
<button>
    Add
</button>
</form>
<div>
    <p>Your current wallet address: {"nil"}</p>
</div>
<div>
    <h2>Tokens</h2>
    <p>You have a total of {0} open source tokens  </p>
    <button>Donate tokens</button>
    <h3>Your last contributions</h3>
</div>
<div>
    Check out these repositories to earn your open source tokens 
</div>
<button on:click={() => PostStatus()}>
Test post status
</button>
</main>