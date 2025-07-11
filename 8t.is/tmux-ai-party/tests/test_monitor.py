#!/usr/bin/env python3
"""
Test suite for Tmux AI Monitor
Trisha's favorite part - making sure everything works perfectly!
As she always says, "Numbers don't lie, but untested code definitely does!" 🧮
"""

import pytest
import os
import sys
from unittest.mock import Mock, patch, MagicMock, AsyncMock
from tmux_monitor import TmuxAIMonitor
import asyncio

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


class TestTmuxAIMonitor:
    """Test cases for our brilliant AI monitor - now with multi-AI support!"""

    @pytest.fixture
    def mock_openai_monitor(self):
        """Create a mock monitor instance using OpenAI"""
        with patch("tmux_monitor.libtmux.Server"):
            with patch("tmux_monitor.OpenAI"):
                with patch("tmux_monitor.genai"):
                    monitor = TmuxAIMonitor(
                        "test-session", "openai", "fake-openai-key", None
                    )
                    # Setup the mocks for async methods
                    monitor._get_openai_summary = AsyncMock(
                        return_value="OpenAI test summary"
                    )
                    monitor._get_openai_next_steps = AsyncMock(
                        return_value="OpenAI next steps"
                    )
                    return monitor

    @pytest.fixture
    def mock_gemini_monitor(self):
        """Create a mock monitor instance using Gemini"""
        with patch("tmux_monitor.libtmux.Server"):
            with patch("tmux_monitor.OpenAI"):
                with patch("tmux_monitor.genai"):
                    monitor = TmuxAIMonitor(
                        "test-session", "gemini", None, "fake-gemini-key"
                    )
                    # Setup the mocks for async methods
                    monitor._get_gemini_summary = AsyncMock(
                        return_value="Gemini test summary"
                    )
                    monitor._get_gemini_next_steps = AsyncMock(
                        return_value="Gemini next steps"
                    )
                    return monitor

    def test_prompt_detection(self, mock_openai_monitor):
        """Test that we correctly detect various prompt patterns"""
        test_cases = [
            (["some output", " > "], True),  # User's example
            (["user@host:~$ "], True),  # Bash prompt
            ([">>> "], True),  # Python prompt
            (["mysql> "], True),  # MySQL prompt
            (["In [1]: "], True),  # IPython prompt
            (["random output"], False),  # No prompt
            ([""], False),  # Empty
        ]

        for lines, expected in test_cases:
            assert mock_openai_monitor.is_at_prompt(lines) == expected

    def test_activity_extraction(self, mock_openai_monitor):
        """Test extraction of activity since last interaction"""
        lines = ["old command", "old output", "new command", "new output", " > "]

        # First call should get all lines
        activity = mock_openai_monitor.get_activity_since_last_interaction(lines)
        assert "old command" in activity
        assert "new output" in activity

        # Set last interaction line
        mock_openai_monitor.last_interaction_line = 2

        # Next call should only get new lines
        activity = mock_openai_monitor.get_activity_since_last_interaction(lines)
        assert "old command" not in activity
        assert "new command" in activity

    def test_config_loading(self, mock_openai_monitor):
        """Test configuration file loading"""
        # Should create default config if not exists
        config = mock_openai_monitor.load_config()
        assert config["openai_summarization_model"] == "gpt-4o"
        assert config["gemini_summarization_model"] == "gemini-1.5-flash"
        assert config["ollama_summarization_model"] == "llama3.2:3b"
        assert config["check_interval"] == 1.0

    def test_system_prompt_loading(self, mock_openai_monitor):
        """Test system prompt loading"""
        prompt = mock_openai_monitor.load_system_prompts()
        assert "AI assistant" in prompt['default']
        assert "terminal" in prompt['default']

    @pytest.mark.asyncio
    async def test_openai_summarization(self, mock_openai_monitor):
        """Test activity summarization with OpenAI"""
        summary = await mock_openai_monitor.summarize_activity("test activity")
        assert summary == "OpenAI test summary"

    @pytest.mark.asyncio
    async def test_openai_next_step_generation(self, mock_openai_monitor):
        """Test next step generation with OpenAI"""
        next_step = await mock_openai_monitor.generate_next_step("summary of activity")
        assert next_step == "OpenAI next steps"

    @pytest.mark.asyncio
    async def test_gemini_summarization(self, mock_gemini_monitor):
        """Test activity summarization with Gemini"""
        summary = await mock_gemini_monitor.summarize_activity("test activity")
        assert summary == "Gemini test summary"

    @pytest.mark.asyncio
    async def test_gemini_next_step_generation(self, mock_gemini_monitor):
        """Test next step generation with Gemini"""
        next_step = await mock_gemini_monitor.generate_next_step("summary of activity")
        assert next_step == "Gemini next steps"

    @pytest.mark.asyncio
    async def test_ollama_summarization(self, mock_ollama_monitor):
        """Test activity summarization with Ollama - Trisha's favorite!"""
        summary = await mock_ollama_monitor.summarize_activity("test activity")
        assert summary == "Ollama test summary"

    @pytest.mark.asyncio
    async def test_ollama_next_step_generation(self, mock_ollama_monitor):
        """Test next step generation with Ollama - budget-friendly AI!"""
        next_step = await mock_ollama_monitor.generate_next_step("summary of activity")
        assert next_step == "Ollama next steps"

    @pytest.fixture
    def mock_ollama_monitor(self):
        """Create a mock monitor instance using Ollama"""
        with patch("tmux_monitor.libtmux.Server"):
            with patch("tmux_monitor.ollama.list") as mock_list:
                # Mock the list response for Ollama
                mock_list.return_value = {"models": [{"name": "llama3.2:3b"}]}
                monitor = TmuxAIMonitor("test-session", "ollama", None, None)
                # Setup the mocks for async methods
                monitor._get_ollama_summary = AsyncMock(
                    return_value="Ollama test summary"
                )
                monitor._get_ollama_next_steps = AsyncMock(
                    return_value="Ollama next steps"
                )
                return monitor

    def test_ai_provider_validation(self):
        """Test that invalid AI providers are rejected"""
        with pytest.raises(ValueError, match="Unsupported AI provider"):
            with patch("tmux_monitor.libtmux.Server"):
                TmuxAIMonitor("test-session", "unsupported_provider", "fake-key", None)

    def test_missing_api_key_validation(self):
        """Test that missing API keys are caught"""
        with pytest.raises(ValueError, match="OpenAI API key is required"):
            with patch("tmux_monitor.libtmux.Server"):
                TmuxAIMonitor("test-session", "openai", None, None)

        with pytest.raises(ValueError, match="Google Gemini API key is required"):
            with patch("tmux_monitor.libtmux.Server"):
                TmuxAIMonitor("test-session", "gemini", None, None)


if __name__ == "__main__":
    # Run tests with pretty output
    pytest.main([__file__, "-v", "--color=yes"])
