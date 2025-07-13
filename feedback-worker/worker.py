#!/usr/bin/env python3
"""
Smart Tree Feedback Worker
Mean ass efficient feedback processor that turns user suggestions into GitHub issues
"""

import asyncio
import json
import os
import re
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Set, Tuple
import hashlib

import aiohttp
import redis.asyncio as redis
from github import Github, GithubException
import uvloop
from prometheus_client import Counter, Histogram, start_http_server
import logging

# Metrics
feedback_processed = Counter('feedback_items_processed_total', 'Total feedback items processed')
feedback_errors = Counter('feedback_errors_total', 'Total errors processing feedback')
github_issues_created = Counter('github_issues_created_total', 'Total GitHub issues created')
duplicate_detected = Counter('feedback_duplicates_detected_total', 'Duplicate feedback detected')
processing_time = Histogram('feedback_processing_seconds', 'Time spent processing feedback')

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('feedback-worker')

class FeedbackWorker:
    def __init__(self):
        self.github_token = os.environ.get('GITHUB_TOKEN')
        self.feedback_api_url = os.environ.get('FEEDBACK_API_URL', 'https://f.8t.is/api')
        self.redis_url = os.environ.get('REDIS_URL', 'redis://localhost:6379')
        self.github_repo = os.environ.get('GITHUB_REPO', '8b-is/smart-tree')
        
        if not self.github_token:
            raise ValueError("GITHUB_TOKEN environment variable required")
        
        self.gh = Github(self.github_token)
        self.repo = self.gh.get_repo(self.github_repo)
        self.redis = None
        self.session = None
        
        # Category patterns
        self.bug_patterns = [
            r'error', r'crash', r'fail', r'broken', r'bug', r'issue',
            r'doesn\'t work', r'not working', r'exception', r'panic'
        ]
        
        self.feature_patterns = [
            r'add', r'feature', r'enhance', r'improve', r'support',
            r'would be nice', r'suggestion', r'idea', r'request'
        ]
        
        self.teleportation_patterns = [
            r'quantum', r'ai', r'machine learning', r'neural',
            r'consciousness', r'teleport', r'magic', r'revolutionary'
        ]
    
    async def setup(self):
        """Initialize connections"""
        self.redis = await redis.from_url(self.redis_url)
        self.session = aiohttp.ClientSession()
        
        # Start metrics server
        start_http_server(int(os.environ.get('PROMETHEUS_PORT', '9090')))
        
        logger.info("Worker initialized")
    
    async def teardown(self):
        """Cleanup connections"""
        if self.session:
            await self.session.close()
        if self.redis:
            await self.redis.close()
    
    def categorize_feedback(self, feedback: Dict) -> str:
        """Categorize feedback as bug, feature, or teleportation goal"""
        text = f"{feedback.get('title', '')} {feedback.get('description', '')}".lower()
        
        # Check for bug patterns
        for pattern in self.bug_patterns:
            if re.search(pattern, text):
                return 'bug'
        
        # Check for teleportation patterns
        for pattern in self.teleportation_patterns:
            if re.search(pattern, text):
                return 'teleportation'
        
        # Check for feature patterns
        for pattern in self.feature_patterns:
            if re.search(pattern, text):
                return 'feature'
        
        # Default to feature
        return 'feature'
    
    def generate_fingerprint(self, feedback: Dict) -> str:
        """Generate fingerprint for duplicate detection"""
        # Normalize text
        text = f"{feedback.get('title', '')} {feedback.get('description', '')}".lower()
        text = re.sub(r'\s+', ' ', text)  # Normalize whitespace
        text = re.sub(r'[^\w\s]', '', text)  # Remove punctuation
        
        # Extract key terms
        important_words = []
        for word in text.split():
            if len(word) > 3 and word not in ['that', 'this', 'with', 'from', 'have']:
                important_words.append(word)
        
        # Create fingerprint
        fingerprint_text = ' '.join(sorted(important_words[:10]))
        return hashlib.sha256(fingerprint_text.encode()).hexdigest()[:16]
    
    async def check_duplicate(self, feedback: Dict) -> Optional[int]:
        """Check if feedback is duplicate, return issue number if found"""
        fingerprint = self.generate_fingerprint(feedback)
        
        # Check Redis cache first
        cached_issue = await self.redis.get(f"feedback:fingerprint:{fingerprint}")
        if cached_issue:
            duplicate_detected.inc()
            return int(cached_issue)
        
        # Search GitHub issues
        try:
            # Search by title similarity
            search_query = feedback.get('title', '').split()[:5]
            search_query = ' '.join(search_query)
            
            issues = self.repo.get_issues(state='all')
            for issue in issues:
                if issue.pull_request:
                    continue
                
                # Check title similarity
                issue_fingerprint = self.generate_fingerprint({
                    'title': issue.title,
                    'description': issue.body or ''
                })
                
                if issue_fingerprint == fingerprint:
                    # Cache the match
                    await self.redis.setex(
                        f"feedback:fingerprint:{fingerprint}",
                        86400,  # 24 hours
                        str(issue.number)
                    )
                    duplicate_detected.inc()
                    return issue.number
        
        except Exception as e:
            logger.error(f"Error checking duplicates: {e}")
        
        return None
    
    async def create_github_issue(self, feedback: Dict, category: str) -> Optional[int]:
        """Create GitHub issue from feedback"""
        try:
            # Check for duplicates
            duplicate_issue = await self.check_duplicate(feedback)
            if duplicate_issue:
                logger.info(f"Duplicate detected: #{duplicate_issue}")
                return duplicate_issue
            
            # Prepare labels
            labels = [category, 'feedback', 'from-ai']
            if feedback.get('impact_score', 0) >= 8:
                labels.append('high-priority')
            
            # Create issue body
            body = f"""## Feedback from {feedback.get('mcp_tool', 'Smart Tree User')}

{feedback.get('description', '')}

### Details
- **Category**: {feedback.get('category', 'Unknown')}
- **Impact Score**: {feedback.get('impact_score', 'N/A')}/10
- **Frequency Score**: {feedback.get('frequency_score', 'N/A')}/10
- **Affected Command**: {feedback.get('affected_command', 'N/A')}

### Examples
"""
            
            # Add examples if provided
            examples = feedback.get('examples', [])
            if examples:
                for example in examples:
                    body += f"\n#### {example.get('description', 'Example')}\n"
                    body += f"```\n{example.get('code', '')}\n```\n"
                    if example.get('expected_output'):
                        body += f"Expected output:\n```\n{example['expected_output']}\n```\n"
            else:
                body += "\nNo examples provided.\n"
            
            # Add proposed solution if available
            if feedback.get('proposed_solution'):
                body += f"\n### Proposed Solution\n{feedback['proposed_solution']}\n"
            
            # Add metadata
            body += f"\n---\n*Automatically created by Smart Tree Feedback Worker*"
            
            # Create the issue
            issue = self.repo.create_issue(
                title=feedback.get('title', 'Untitled Feedback'),
                body=body,
                labels=labels
            )
            
            github_issues_created.inc()
            logger.info(f"Created issue #{issue.number}: {issue.title}")
            
            # Cache the fingerprint
            fingerprint = self.generate_fingerprint(feedback)
            await self.redis.setex(
                f"feedback:fingerprint:{fingerprint}",
                86400,  # 24 hours
                str(issue.number)
            )
            
            return issue.number
            
        except GithubException as e:
            logger.error(f"GitHub API error: {e}")
            feedback_errors.inc()
            return None
        except Exception as e:
            logger.error(f"Error creating issue: {e}")
            feedback_errors.inc()
            return None
    
    async def process_feedback(self, feedback: Dict):
        """Process a single feedback item"""
        with processing_time.time():
            try:
                # Categorize feedback
                category = self.categorize_feedback(feedback)
                logger.info(f"Processing {category}: {feedback.get('title', 'Untitled')}")
                
                # Create GitHub issue
                issue_number = await self.create_github_issue(feedback, category)
                
                if issue_number:
                    # Mark as processed in API
                    async with self.session.post(
                        f"{self.feedback_api_url}/feedback/{feedback['id']}/processed",
                        json={
                            'github_issue': issue_number,
                            'category': category,
                            'processed_at': datetime.utcnow().isoformat()
                        }
                    ) as resp:
                        if resp.status != 200:
                            logger.error(f"Failed to mark feedback as processed: {resp.status}")
                
                feedback_processed.inc()
                
            except Exception as e:
                logger.error(f"Error processing feedback: {e}")
                feedback_errors.inc()
                
                # Store in Redis for retry
                await self.redis.lpush(
                    'feedback:retry',
                    json.dumps(feedback)
                )
    
    async def fetch_pending_feedback(self) -> List[Dict]:
        """Fetch pending feedback from API"""
        try:
            async with self.session.get(f"{self.feedback_api_url}/feedback/pending") as resp:
                if resp.status == 200:
                    return await resp.json()
                else:
                    logger.error(f"Failed to fetch feedback: {resp.status}")
                    return []
        except Exception as e:
            logger.error(f"Error fetching feedback: {e}")
            return []
    
    async def run(self):
        """Main worker loop"""
        await self.setup()
        
        try:
            while True:
                # Check for retry items first
                retry_item = await self.redis.rpop('feedback:retry')
                if retry_item:
                    feedback = json.loads(retry_item)
                    await self.process_feedback(feedback)
                    continue
                
                # Fetch new feedback
                feedback_items = await self.fetch_pending_feedback()
                
                if feedback_items:
                    logger.info(f"Processing {len(feedback_items)} feedback items")
                    
                    # Process concurrently but with limit
                    semaphore = asyncio.Semaphore(3)
                    
                    async def process_with_limit(item):
                        async with semaphore:
                            await self.process_feedback(item)
                    
                    await asyncio.gather(*[
                        process_with_limit(item) for item in feedback_items
                    ])
                
                # Wait before next poll
                await asyncio.sleep(30)
                
        except KeyboardInterrupt:
            logger.info("Shutting down...")
        finally:
            await self.teardown()

async def main():
    """Entry point"""
    # Use uvloop for better performance
    asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
    
    worker = FeedbackWorker()
    await worker.run()

if __name__ == '__main__':
    asyncio.run(main())