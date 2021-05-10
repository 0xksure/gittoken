const API_URL="http://localhost:8000/v0"


export async function fetchAPI<T>(path: string,method:string ="GET",body:Record<string,any>=undefined): Promise<any>{
      
    const resp = await fetch(`${API_URL}${path}`, {
        body: body ? JSON.stringify(body) : undefined,
        method: method,
        headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
        },
        credentials: 'include',
    })
    if (resp.status >= 300){
        throw new Error(`unexpected status: ${resp.status}`)
        
    }
    if (resp.body){
        try{
            const data = await resp.json()
            return data
        }catch{
            return {}
        }
    }
   
    return {}
}