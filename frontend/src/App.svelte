<script lang="ts">
  	import { Router, Link, Route } from "svelte-routing";
	import Landing from "./Landing.svelte"
	import Login from "./Login.svelte"
  import Home from "./portal/Home.svelte"
  import Install from "./public/Install.svelte"
	export let url: String = "";
</script>



<main>
	<Router url="{url}">
  <div>
    <Route path="" component="{Home}" />
    <Route path="login" component="{Login}" />
      <Route path="home" component="{Home}" />
      <Route path="install" component="{Install}" />
    <Route path="/" component="{Landing}"/>
  </div>
</Router>
</main>
