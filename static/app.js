// app.js
class AudioAgent {
    constructor() {
      this.mediaRecorder = null;
      this.audioChunks = [];
      this.isRecording = false;
      this.stream = null;
      this.recordingStartTime = null;
      this.processingStartTime = null;
  
      this.conversationHistory = [];
      this.maxHistoryLength = 10;
  
      this.player = null;          // single reusable audio element
      this.audioResponse = null;   // Uint8Array from server
  
      this.initElements();
      this.initEvents();
    }
  
    /* ---------- DOM refs ---------- */
    initElements() {
      this.recordToggleButton = document.getElementById('recordToggleButton');
      this.playResponseButton = document.getElementById('playResponse');
      this.responseText       = document.getElementById('responseText');
      this.processingTime     = document.getElementById('processingTime');
      this.audioDuration      = document.getElementById('audioDuration');
      this.statusDot          = document.querySelector('.status-dot');
      this.statusText         = document.querySelector('.status-text');
      this.processingIndicator= document.getElementById('processingIndicator');
    }
  
    initEvents() {
      this.recordToggleButton?.addEventListener('click', () => this.toggleRecording());
      this.playResponseButton?.addEventListener('click',  () => this.playResponse());
    }
  
    /* ---------- record control ---------- */
    async toggleRecording() {
      if (!this.isRecording) this.stopResponse();     // stop any playback first
      this.isRecording ? this.stopRecording() : await this.startRecording();
    }
  
    async startRecording() {
      try {
        this.stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        this.mediaRecorder = new MediaRecorder(this.stream);
        this.audioChunks = [];
        this.recordingStartTime = Date.now();
  
        this.mediaRecorder.ondataavailable = e => e.data.size && this.audioChunks.push(e.data);
        this.mediaRecorder.onstop = () => this.processAudio();
  
        this.mediaRecorder.start();
        this.updateUI(true);
        this.updateStatus('recording', 'Recording...');
      } catch (err) {
        console.error(err);
        this.updateStatus('error', 'Failed to start recording');
      }
    }
  
    stopRecording() {
      if (!this.mediaRecorder || !this.isRecording) return;
      this.mediaRecorder.stop();
      this.stream.getTracks().forEach(t => t.stop());
      this.updateUI(false);
      this.updateStatus('processing', 'Processing...');
    }
  
    /* ---------- server ---------- */
    async processAudio() {
      this.processingStartTime = Date.now();
      const blob = new Blob(this.audioChunks, { type: this.mediaRecorder.mimeType });
      const fd = new FormData();
      fd.append('audio', blob, 'recording.webm');
      fd.append('conversation_history', JSON.stringify(this.conversationHistory));
  
      this.processingIndicator.style.display = 'block';
  
      try {
        const res = await fetch('http://127.0.0.1:3000/process_voice_agent', {
          method: 'POST',
          body: fd
        });
        if (!res.ok) throw new Error('Processing failed');
  
        const data = await res.json();
        if (!data.audio_data) throw new Error('No audio data from server');
  
        this.addToHistory({ role: 'user',      content: data.transcription || 'User audio' });
        if (data.text) this.addToHistory({ role: 'assistant', content: data.text });
  
        this.audioResponse = new Uint8Array(data.audio_data);
        this.updateMetrics();
        this.updateStatus('success', 'Processing complete');
  
        this.stopResponse();      // stop any previous playback
        this.playResponse();      // auto-play new response
      } catch (err) {
        console.error(err);
        this.updateStatus('error', err.message);
      } finally {
        this.processingIndicator.style.display = 'none';
      }
    }
  
    addToHistory(msg) {
      this.conversationHistory.push(msg);
      if (this.conversationHistory.length > this.maxHistoryLength)
        this.conversationHistory = this.conversationHistory.slice(-this.maxHistoryLength);
      this.renderHistory();       // refresh the Response box
    }
  
    /* ---------- UI ---------- */
    renderHistory() {
      /* newest first */
      const html = this.conversationHistory
        .slice()                       // copy
        .reverse()                     // newest on top
        .map(m => `<div class="history-item ${m.role}">
                    <strong>${m.role === 'assistant' ? '🤖' : '🗣️'}</strong>
                    <span>${m.content}</span>
                   </div>`)
        .join('');
      this.responseText.innerHTML = html;
      this.playResponseButton.disabled = false;
    }
  
    updateMetrics() {
      const proc = Date.now() - this.processingStartTime;
      const dur  = (Date.now() - this.recordingStartTime) / 1000;
      this.processingTime.textContent = `${proc} ms`;
      this.audioDuration.textContent  = `${dur.toFixed(1)} s`;
    }
  
    /* keeps existing .button class; only toggles primary/secondary/recording */
    updateUI(isRecording) {
      this.isRecording = isRecording;
  
      if (isRecording) {
        this.recordToggleButton.innerHTML =
          '<i class="fas fa-stop"></i><span>Stop Recording</span>';
        this.recordToggleButton.classList.remove('primary');
        this.recordToggleButton.classList.add('secondary', 'recording');
      } else {
        this.recordToggleButton.innerHTML =
          '<i class="fas fa-microphone"></i><span>Start Recording</span>';
        this.recordToggleButton.classList.remove('secondary', 'recording');
        this.recordToggleButton.classList.add('primary');
      }
    }
  
    updateStatus(type, msg) {
      this.statusText.textContent = msg;
      this.statusDot.style.backgroundColor = {
        recording:  '#EF4444',
        processing: '#F59E0B',
        success:    '#10B981',
        error:      '#EF4444'
      }[type];
    }
  
    /* ---------- playback ---------- */
    stopResponse() {
      if (this.player) {
        this.player.pause();
        this.player.currentTime = 0;
      }
      this.updateStatus('success', 'Playback stopped');
    }
  
    playResponse() {
      if (!this.audioResponse) {
        this.updateStatus('error', 'No audio response available');
        return;
      }
      if (!this.player) this.player = new Audio();
  
      const url = URL.createObjectURL(
        new Blob([this.audioResponse], { type: 'audio/wav' })
      );
  
      this.player.src = url;
      this.player.onended = () => URL.revokeObjectURL(url);
      this.player.onerror = () => {
        URL.revokeObjectURL(url);
        this.updateStatus('error', 'Failed to play response');
      };
  
      this.player.play()
        .then(() => this.updateStatus('success', 'Playing response...'))
        .catch(err => {
          console.error(err);
          this.updateStatus('error', 'Failed to play response');
        });
    }
  }
  
  document.addEventListener('DOMContentLoaded', () => new AudioAgent());
  