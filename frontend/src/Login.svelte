<script lang="ts">
    import {fetchAPI} from "./api/api"
    import queryString from "query-string";
    let error_message ="";
   let parsed = {};
  if (typeof window !== 'undefined') {
    parsed = queryString.parse(window.location.search);
    console.log("parsed: ",parsed)
  }

    async function fetcher(){
    try{
        const resp = await fetchAPI(`/github/login?code=${parsed.code}`,"GET")
        console.log(resp)
        location.href="/home"
      }catch(err){
          error_message = err
          console.log("error: ",err)
          location.href ="faile"
      }
    }

  if(parsed.code){
      console.log("code passed")
     fetcher()
  }
</script>

<main>
    Processing...
</main>