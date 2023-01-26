<template>
  <div class="app-wrapper">
    <div class="container">
      <div v-if="connecting" class="connecting">
        <h1>Connecting to the lamp...</h1>
      </div>

      <div class="top-bar">
        <div ref="preview" class="preview"/>
      </div>

      <div class="picker" ref="picker">
        <color-column
          ref="hue"
          v-model:value="hue"
          :min="0"
          :max="360"
          :gradient="generateHueGradient"
          @end="sendColor(['hue'], true)"
          @update:value="cycling = false"
        >
          <template #top-button>
            <div class="top-button" @click="cycle()">
              Cycle
            </div>
          </template>
        </color-column>

        <color-column
          ref="sat"
          v-model:value="sat"
          :min="0"
          :max="100"
          :gradient="generateSatGradient"
          with-buttons
          @min="sat = 0"
          @max="sat = 100"
          @end="sendColor(['sat'], true)"
        />

        <color-column
          ref="val"
          v-model:value="val"
          :min="0"
          :max="100"
          :gradient="generateValGradient"
          with-buttons
          @min="val = 0"
          @max="val = 100"
          @end="sendColor(['val'], true)"
        />
      </div>
    </div>

    <div class="container connect-page">
      <div class="inputs">
        <input placeholder="Wifi credentials" v-model="wifi.ssid"/>
        <input placeholder="Wifi password" type="password" v-model="wifi.password"/>
      </div>

      <div class="buttons">
        <button v-if="!connecting" @click="resetWifiCredentials()">Forget Wifi network</button>
        <button @click="sendWifiCredentials()">Connect to Wifi</button>
      </div>
    </div>
  </div>
</template>

<script>
import Color from 'color'
import * as TauriApi from '@tauri-apps/api'
import ColorColumn from './components/color-column.vue'

const SEND_FREQ = 200

export default {
  data() {
    return {
      lastSendAt: 0,
      dummyCtx: null,
      hue: 0,
      sat: 0,
      val: 0,
      color: null,
      hueColor: null,
      cycling: false,
      startingHue: null,
      lastTick: 0,
      wifi: {
        ssid: '',
        password: '',
      }
    }
  },
  created() {
    this.updateColor()

    this.connectToLamp()
  },
  mounted() {
    this.$refs.preview.style.background = this.color

    window.addEventListener('resize', () => {
      this.$refs.hue.init()
      this.$refs.sat.init()
      this.$refs.val.init()
    })
  },
  methods: {
    generateHueGradient(gradient) {
      for(let i=1; i<360; ++i) {
        let color = `hsl(${i}, ${100}%, ${50}%)`
        gradient.addColorStop(i/360, color)
      }
    },
    generateSatGradient(gradient) {
      gradient.addColorStop(0, '#fff')
      gradient.addColorStop(1, this.hueColor)
    },
    generateValGradient(gradient) {
      gradient.addColorStop(0, '#000')
      gradient.addColorStop(1, this.hueColor)
    },
    updateColor() {
      this.color = Color.hsv(this.hue, this.sat, this.val)
      this.hueColor = Color.hsv(this.hue, 100, 100)
    },
    sendColor(components, force = false) {
      let now = Date.now()
      if(!force && now - this.lastSendAt < SEND_FREQ) return
      this.lastSendAt = now

      let toSend = {}
      for(let component of components) {
        switch(component) {
          case 'hue': toSend.hue = this.hue; break;
          case 'sat': toSend.sat = this.sat / 100; break;
          case 'val': toSend.val = this.val / 100; break;
        }
      }

      TauriApi.invoke('send_color', {components: toSend})
        .catch( err => console.error(err) )
    },
    cycleColor(duration) {
      let calcFrame = () => {
        if(!this.cycling) return

        let elapsedSecs = (Date.now() - this.lastTick) / 1000
        let progress = elapsedSecs / duration
        this.hue = (this.startingHue + progress * 360) % 360

        requestAnimationFrame(calcFrame)
      }

      this.cycling = true
      this.startingHue = this.hue
      this.lastTick = Date.now()

      requestAnimationFrame(calcFrame)
    },
    connectToLamp() {
      this.connecting = true

      TauriApi.invoke('connect')
        .then( () => TauriApi.invoke('get_status') )
        .then( res => {
          let status = JSON.parse(res)

          this.hue = status.hue
          this.sat = status.sat * 100
          this.val = status.val * 100

          if(status.cycling)
            this.cycleColor(status.duration)
          else
            this.updateColor()

          this.connecting = false
        })
        .catch( err => console.error(err) )
    },
    cycle() {
      TauriApi.invoke("send_cycle")
        .then( res => this.cycleColor(Number(res)) )
        .catch( err => console.error(err) )
    },
    reset() {
      TauriApi.invoke("send_reset")
        .catch( err => console.error(err) )
    },
    sendWifiCredentials() {
      TauriApi.invoke("send_wifi_credentials", this.wifi)
        .then( () => {
          this.wifi.ssid = ''
          this.wifi.password = ''
        })
        .catch( err => console.error(err) )
    },
    resetWifiCredentials() {
      TauriApi.invoke("send_reset")
        .then( () => {
          setTimeout(this.connectToLamp, 1500) // we need to wait 1 sec before the lamp resets
        })
        .catch( err => console.error(err) )
    },
  },
  watch: {
    hue() {
      this.updateColor()

      if(!this.cycling) this.sendColor(['hue'])

      this.$refs.sat.draw()
      this.$refs.val.draw()
      this.$refs.preview.style.background = this.color
    },
    sat() {
      this.updateColor()
      this.sendColor(['sat'])

      this.$refs.preview.style.background = this.color
    },
    val() {
      this.updateColor()
      this.sendColor(['val'])

      this.$refs.preview.style.background = this.color
    },
  },  
  components: {
    ColorColumn
  }
}
</script>

<style scoped>
.app-wrapper {
  height: 100%;
  display: flex;
  overflow: auto;
  scroll-snap-type: x mandatory;
}
.connect-page {
  justify-content: space-between;
}
.connect-page h1 {
  font-size: 25px;
  color: white;
}
.connect-page .inputs {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.connect-page .inputs input {
  border-radius: 10px;
  border: none;
  height: 40px;
  padding: 0 10px;
}
.connect-page .buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.connect-page button {
  border-radius: 10px;
  border: none;
  height: 40px;
}
.connecting {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.8);
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.connecting h1 {
  color: white;
  font-size: 28px;
}
.container {
  scroll-snap-align: start;
  flex-shrink: 0;
  width: 100%;
  height: 100%;
  display: flex;
  margin: 0 auto;
  flex-direction: column;
  gap: 10px;
  overflow: hidden;
  padding: 10px;
  background: black;
}
.top-bar {
  flex-shrink: 0;
  height: 100px;
  display: flex;
  gap: 10px;
}
.preview {
  flex: 1;
  border: 2px solid white;
  border-radius: 10px;
}
.picker {
  width: 100%;
  overflow: hidden;
  flex: 1;
  display: flex;
  gap: 10px;
}
</style>