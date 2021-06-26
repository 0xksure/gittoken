<script lang="ts">
  import { onMount } from 'svelte'
  import { fetchAPI } from '../api/api'
  import { User } from '../state/index'
  import {
    Connection,
    SystemProgram,
    Transaction,
    clusterApiUrl
  } from '@solana/web3.js/lib/index.esm.js'
  import Wallet from '@project-serum/sol-wallet-adapter/dist/index'

  let user: User = { username: '', name: '' }
  let github_profile: String = ''
  let account_balance: Number = -1
  let public_address: String
  let wallet: Wallet
  let connection: Connection
  let wallet_connection_success: boolean = false

  // Get user information
  onMount(async () => {
    try {
      const resp = await fetchAPI(`/user`)
      user = resp
      console.log(resp)
    } catch (err) {
      throw new Error(err)
    }
  })

  async function PostStatus() {
    try {
      const resp = await fetchAPI(`/github/app/post/status`)
      user = resp
      console.log('Status: ', resp)
    } catch (err) {
      throw new Error(err)
    }
  }

  async function connectWallet() {
    connection = new Connection(clusterApiUrl('devnet'))
    let providerUrl = 'https://www.sollet.io'
    const network = clusterApiUrl('devnet', false)
    let wallet = new Wallet(providerUrl, network)

    wallet.on('connect', async publicKey => {
      public_address = publicKey.toBase58()

      account_balance = await connection
        .getBalance(publicKey)
        .then(bal => {
          console.log('balance: ', bal)
          return bal
        })
        .catch(err => console.log('err: ', err))
      console.log('Connected to ' + public_address)
      wallet_connection_success = true
    })

    wallet.on('disconnect', () => {
      wallet_connection_success = false
    })
    await wallet.connect()
  }

  async function sendTransaction() {
    let transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: wallet.publicKey,
        lamports: 100
      })
    )

    let { blockhash } = await connection.getRecentBlockhash()
    transaction.recentBlockhash = blockhash
    transaction.feePayer = wallet.publicKey
    let signed = await wallet.signTransaction(transaction)
    let txid = await connection.sendRawTransaction(signed.serialize())
    await connection.confirmTransaction(txid)
  }
</script>

<div>
  <h1>Open source token</h1>
  <h3>Github profile {'user?.username'}</h3>
  <button on:click={() => connectWallet()}>Connect wallet</button>
  {#if wallet_connection_success}
    <div>
      <p>Your current wallet address: {public_address}</p>
    </div>
  {/if}
  <div>
    <h2>Tokens</h2>
    {#if account_balance >= 0}
      <p>You have a total of {account_balance} open source tokens</p>
    {:else}
      <p>Connect wallet to load account balance</p>
    {/if}

    <button>Transfer tokens</button>
    <h3>Your last contributions</h3>
  </div>
  <div>
    <h3>Pull request transactions ready to be approved</h3>
  </div>
  <div>Check out these repositories to earn your open source tokens</div>
  <button on:click={() => PostStatus()}> Test post status </button>
</div>
